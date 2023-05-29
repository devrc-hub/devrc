use crate::{
    config::RawConfig,
    environment::Environment,
    errors::{DevrcError, DevrcResult},
    include::Include,
    loader::LoadingConfig,
    resolver::Location,
    tasks::Task,
};

use std::{fmt::Debug, str::FromStr};

use serde::Deserialize;

use crate::{
    de::deserialize_some, env_file::EnvFilesInclude, environment::RawEnvironment, tasks::Tasks,
    variables::RawVariables,
};

#[derive(Debug, Deserialize, Clone)]
pub struct GitlabCIConfig {}

fn default_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Clone, Default)]
pub enum Kind {
    #[default]
    None,
    Directory,
    DirectoryLocal,
    Args,
    Global,
    Include,
    StdIn,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RawDevrcfile {
    #[allow(dead_code)]
    #[serde(default = "default_version")]
    version: String,

    #[serde(default)]
    pub environment: RawEnvironment<String>,

    // Environment variables from files
    #[serde(skip_deserializing)]
    pub files_environment: Environment<String>,

    #[serde(default)]
    pub variables: RawVariables,

    #[serde(default)]
    #[serde(rename(deserialize = "include"))]
    pub include: Vec<Include>,

    // #[serde(default)]
    #[serde(rename(deserialize = "env_file"))]
    pub envs_files: Option<EnvFilesInclude>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub after_script: Option<Option<Task>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub before_script: Option<Option<Task>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub before_task: Option<Option<Task>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub after_task: Option<Option<Task>>,

    #[serde(default)]
    #[serde(rename(deserialize = "devrc_config"))]
    pub config: RawConfig,

    #[serde(flatten)]
    pub tasks: Tasks,

    #[serde(skip_deserializing)]
    pub location: Location,

    #[serde(skip_deserializing)]
    pub kind: Kind,
}

impl RawDevrcfile {
    pub fn with_location(self, location: Location) -> Self {
        Self { location, ..self }
    }

    pub fn get_tasks(&self) -> &Tasks {
        &self.tasks
    }

    pub fn is_global_enabled(&self) -> bool {
        if let Some(use_global) = self.config.global {
            return use_global;
        }
        false
    }

    pub fn prepared_from_str(
        content: &str,
        location: Location,
        kind: Kind,
        loading_config: LoadingConfig,
    ) -> DevrcResult<Self> {
        match Self::from_str(content) {
            Ok(value) => value
                .with_location(location)
                .with_kind(kind)
                .prepared(loading_config),
            Err(error) => Err(error),
        }
    }

    pub fn load_env_files(&mut self, loading_config: LoadingConfig) -> DevrcResult<()> {
        if let Some(files) = &self.envs_files {
            let env_vars = files.load(self.location.clone(), loading_config)?;

            for (key, value) in env_vars {
                self.files_environment.insert(key, value);
            }
        }
        Ok(())
    }

    pub fn prepare(&mut self, loading_config: LoadingConfig) -> DevrcResult<()> {
        self.load_env_files(loading_config)
    }

    pub fn with_kind(self, kind: Kind) -> Self {
        Self { kind, ..self }
    }

    pub fn setup_kind(&mut self, kind: Kind) {
        self.kind = kind
    }

    pub fn prepared(mut self, loading_config: LoadingConfig) -> DevrcResult<RawDevrcfile> {
        self.prepare(loading_config)?;
        Ok(self)
    }
}

impl FromStr for RawDevrcfile {
    type Err = DevrcError;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str::<RawDevrcfile>(content).map_err(DevrcError::YamlParseError)

        // let config: Self = match serde_yaml::from_str::<RawDevrcfile>(content) {
        //     Ok(mut value) => {
        //         value.setup_location(Location::LocalFile(PathBuf::from("/dev/stdin")));
        //         value
        //     },
        //     Err(error) => return Err(DevrcError::YamlParseError(error)),
        // };
        // Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::RawDevrcfile;

    #[test]
    fn test_rawdevrcfile() {
        let _devrcfile = RawDevrcfile::default();
    }
}
