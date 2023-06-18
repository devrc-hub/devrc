use std::{convert::TryFrom, env, fs, path::PathBuf};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use sha256::digest;
use url::Url;

use crate::{
    de::deserialize_some,
    environment::Environment,
    errors::{DevrcError, DevrcResult},
    loader::LoadingConfig,
    resolver::{Location, PathResolve},
    utils,
};

pub(crate) fn get_default_skip_on_error() -> bool {
    false
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LocalFileImport {
    pub file: PathBuf,

    #[serde(default = "get_default_skip_on_error")]
    pub ignore_errors: bool,

    #[serde(default)]
    pub path_resolve: PathResolve,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub checksum: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UrlImport {
    pub url: String,

    #[serde(default = "get_default_skip_on_error")]
    pub ignore_errors: bool,

    pub checksum: String,

    #[serde(default)]
    pub headers: indexmap::IndexMap<String, String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(untagged)]
pub enum EnvFilesInclude {
    #[default]
    Empty,
    Simple(PathBuf),
    File(LocalFileImport),
    Url(UrlImport),
    List(Vec<EnvFilesInclude>),
}

impl From<&PathBuf> for LocalFileImport {
    fn from(source: &PathBuf) -> Self {
        Self {
            file: (*source).clone(),
            ..Default::default()
        }
    }
}

impl EnvFilesInclude {
    pub fn load(
        &self,
        location: Location,
        config: LoadingConfig,
    ) -> DevrcResult<Environment<String>> {
        match self {
            EnvFilesInclude::Empty => Ok(Default::default()),
            EnvFilesInclude::Simple(path) => LocalFileImport::from(path).load(location, config),
            EnvFilesInclude::File(file_include) => file_include.load(location, config),
            EnvFilesInclude::Url(remote_file) => remote_file.load(location, config),
            EnvFilesInclude::List(list) => {
                let mut env: Environment<String> = Default::default();
                for include in list {
                    for (key, value) in include.load(location.clone(), config.clone())? {
                        env.insert(key, value);
                    }
                }

                Ok(env)
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnvFilesWrapper(pub EnvFilesInclude);

pub trait Loader {
    fn load(&self, location: Location, config: LoadingConfig) -> DevrcResult<Environment<String>>;
}

pub fn read_env_from_string(input: &str) -> DevrcResult<Environment<String>> {
    let mut environment = Environment::default();
    for item in dotenvy::Iter::new(input.as_bytes()) {
        let (key, value) = item.map_err(DevrcError::Dotenv)?;
        environment.insert(key, value);
    }
    Ok(environment)
}

impl Loader for LocalFileImport {
    fn load(&self, location: Location, config: LoadingConfig) -> DevrcResult<Environment<String>> {
        let environment = if self.ignore_errors {
            match self.get_content(location, config) {
                Ok(content) => read_env_from_string(&content).unwrap_or_default(),
                Err(_error) => Environment::default(),
            }
        } else {
            read_env_from_string(&self.get_content(location, config)?)?
        };
        Ok(environment)
    }
}

impl LocalFileImport {
    fn get_content(&self, location: Location, config: LoadingConfig) -> DevrcResult<String> {
        let loading_location = self.get_loading_location(location, &config)?;

        config.log_level.debug(
            &format!("\n==> Loading ENV FILE: `{:}` ...", &loading_location),
            &config.designer.banner(),
        );

        match loading_location {
            Location::LocalFile(path) => fs::read_to_string(path).map_err(DevrcError::IoError),
            Location::Remote { url, auth } => {
                if let Some(cache_ttl) = config.cache_ttl {
                    if let Some(content) = crate::cache::load(&url, &config, None, &cache_ttl) {
                        config.log_level.debug(
                            &format!("\n==> Loading ENV URL CACHE: `{}` ...", &url),
                            &config.designer.banner(),
                        );
                        return Ok(content);
                    }
                }

                let client = reqwest::blocking::Client::new();
                let mut headers_map: HeaderMap = HeaderMap::new();

                if let Some((key, value)) = auth.get_header() {
                    headers_map.insert(
                        HeaderName::try_from(key.clone()).map_err(|_| {
                            DevrcError::UrlImportHeadersError {
                                name: key.clone(),
                                value: value.clone(),
                            }
                        })?,
                        HeaderValue::try_from(value.clone()).map_err(|_| {
                            DevrcError::UrlImportHeadersError {
                                name: key.clone(),
                                value: value.clone(),
                            }
                        })?,
                    );
                }

                match client.get(url.as_str()).headers(headers_map).send() {
                    Ok(response) if response.status() == 200 => {
                        let content = response.text().map_err(|_| DevrcError::RuntimeError)?;

                        if let Some(control_checksum) = self.checksum.clone() {
                            let content_checksum = digest(content.as_str());

                            if control_checksum != content_checksum {
                                return Err(DevrcError::UrlImportChecksumError {
                                    url: url.as_str().to_string(),
                                    control_checksum,
                                    content_checksum,
                                });
                            }
                        }

                        Ok(content)
                    }
                    Ok(response) => {
                        config.log_level.debug(
                            &format!(
                                "Loadin ENV FILE error: invalid status code `{:}` ...",
                                response.status()
                            ),
                            &config.designer.banner(),
                        );
                        Err(DevrcError::EnvfileUrlImportStatusError {
                            url: url.as_str().to_string(),
                            status: response.status(),
                        })
                    }
                    Err(error) => {
                        config.log_level.debug(
                            &format!("Error: `{:}` ...", &error),
                            &config.designer.banner(),
                        );
                        Err(DevrcError::RuntimeError)
                    }
                }
            }
            _ => Ok("".to_string()),
        }
    }

    pub fn get_loading_location(
        &self,
        location: Location,
        _config: &LoadingConfig,
    ) -> DevrcResult<Location> {
        if self.file.is_absolute() {
            return Ok(Location::LocalFile(self.file.clone()));
        }

        match location {
            Location::None | Location::StdIn => {
                let path = utils::get_absolute_path(&self.file, None)?;
                Ok(Location::LocalFile(path))
            }
            Location::LocalFile(file) => match self.path_resolve {
                PathResolve::Relative => {
                    let path = utils::get_absolute_path(&self.file, Some(&file))?;
                    Ok(Location::LocalFile(path))
                }
                PathResolve::Pwd => {
                    let path =
                        utils::get_absolute_path(&self.file, env::current_dir().ok().as_ref())?;
                    Ok(Location::LocalFile(path))
                }
            },
            Location::Remote { url, auth } => match self.path_resolve {
                PathResolve::Relative => {
                    let path = self
                        .file
                        .clone()
                        .into_os_string()
                        .into_string()
                        .map_err(|_| DevrcError::RuntimeError)?;
                    let include_url = url.join(&path).map_err(|_| DevrcError::RuntimeError)?;
                    Ok(Location::Remote {
                        url: include_url,
                        auth,
                    })
                }
                PathResolve::Pwd => {
                    let path =
                        utils::get_absolute_path(&self.file, env::current_dir().ok().as_ref())?;
                    Ok(Location::LocalFile(path))
                }
            },
        }
    }
}

impl Loader for UrlImport {
    fn load(&self, location: Location, config: LoadingConfig) -> DevrcResult<Environment<String>> {
        let environment = if self.ignore_errors {
            match self.get_content(location, config) {
                Ok(content) => read_env_from_string(&content).unwrap_or_default(),
                Err(_) => Environment::default(),
            }
        } else {
            read_env_from_string(&self.get_content(location, config)?)?
        };
        Ok(environment)
    }
}

impl UrlImport {
    pub fn get_content(&self, _location: Location, config: LoadingConfig) -> DevrcResult<String> {
        let parsed_url =
            Url::parse(&self.url).map_err(|_| DevrcError::InvalidIncludeUrl(self.url.clone()))?;

        if let Some(cache_ttl) = config.cache_ttl {
            if let Some(content) =
                crate::cache::load(&parsed_url, &config, Some(&self.checksum), &cache_ttl)
            {
                config.log_level.debug(
                    &format!("\n==> Loading ENV URL CACHE: `{}` ...", &parsed_url),
                    &config.designer.banner(),
                );
                return Ok(content);
            }
        }

        config.log_level.debug(
            &format!("\n==> Loading ENV FILE: `{:}` ...", &parsed_url),
            &config.designer.banner(),
        );

        match reqwest::blocking::get(parsed_url.as_str()) {
            Ok(response) if response.status() == 200 => {
                let content = response.text().map_err(|_| DevrcError::RuntimeError)?;

                let content_checksum = digest(content.as_str());

                if self.checksum != content_checksum {
                    return Err(DevrcError::UrlImportChecksumError {
                        url: parsed_url.as_str().to_string(),
                        control_checksum: self.checksum.to_string(),
                        content_checksum,
                    });
                }

                if config.cache_ttl.is_some() {
                    crate::cache::save(&parsed_url, &content)?;
                }

                Ok(content)
            }
            Ok(response) => {
                config.log_level.debug(
                    &format!(
                        "Loadin ENV FILE error: invalid status code `{:}` ...",
                        response.status()
                    ),
                    &config.designer.banner(),
                );
                Err(DevrcError::EnvfileUrlImportStatusError {
                    url: parsed_url.as_str().to_string(),
                    status: response.status(),
                })
            }
            Err(error) => {
                config.log_level.debug(
                    &format!("Error: `{:}` ...", &error),
                    &config.designer.banner(),
                );
                Err(DevrcError::EnvfileUrlImportError {
                    url: parsed_url.as_str().to_string(),
                    inner: error,
                })
            }
        }
    }
}
