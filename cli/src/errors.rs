use devrc_plugins::errors::DevrcPluginError;
use reqwest::StatusCode;
use serde_yaml::Error as SerdeYamlError;
use std::{
    error::Error as StdError,
    fmt,
    fmt::{Display, Formatter},
    io::Error as IoError,
    path::{PathBuf, StripPrefixError},
};

use tera::Error as TeraError;

use dotenvy::{self, Error as DotenvError};

use crate::resolver::Location;

pub type DevrcResult<T> = Result<T, DevrcError>;

#[derive(Debug)]
pub enum DevrcError {
    Dotenv(DotenvError),
    NotExists,
    FileNotExists(PathBuf),
    PluginFileNotExists(PathBuf),
    PluginError(DevrcPluginError),
    GlobalNotExists,
    LocalNotExists,
    RenderError(TeraError),
    EmptyVariable,
    InvalidVariableType,
    VariableTypeNotImplemented,
    EmptyEnvironmentVariable,
    IoError(IoError),
    YamlParseError(SerdeYamlError),
    TaskNotFound,
    NotImplemented,
    Signal,
    Code {
        code: i32,
    },
    CircularDependencies,
    InvalidArgument,
    InvalidName,
    InvalidParams,
    InvalidVariableName,
    InvalidVariableModifier,
    InvalidIncludeUrl(String),
    TaskArgumentsParsingError,
    OverlappingParameters,
    NotEnouthArguments,
    InvalidInterpreter,
    NestingLevelExceed,
    RuntimeError,
    EnvfileImportError {
        location: Location,
    },
    EnvfileUrlImportStatusError {
        url: String,
        status: StatusCode,
    },
    EnvfileUrlImportError {
        url: String,
        inner: reqwest::Error,
    },
    FileImportError,
    UrlImportStatusError {
        url: String,
        status: StatusCode,
    },
    UrlImportRequestError {
        url: String,
        inner: reqwest::Error,
    },
    UrlImportError,
    UrlImportHeadersError {
        name: String,
        value: String,
    },
    UrlImportChecksumError {
        url: String,
        control_checksum: String,
        content_checksum: String,
    },
    AnyhowError(anyhow::Error),
    HomeDirNotFound,
    NetrcNotFound,
    NetrcParsingError(netrc_rs::Error),
}

impl Display for DevrcError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "devrc error: ")?;

        match self {
            // TODO: add source context to error
            DevrcError::RenderError(terra_error) => {
                match TeraError::source(terra_error) {
                    Some(value) => {
                        write!(f, "{:}", &value)?;
                    }
                    _value => {
                        writeln!(f, "another value")?;
                    }
                }
                // write!(f, "{}: ", terra_error);
            }
            DevrcError::Code { code } => {
                write!(f, "Recipe failed with code {:}", code)?;
            }
            DevrcError::FileNotExists(location) => {
                write!(f, "File {:} not found", location.display())?;
            }
            DevrcError::InvalidIncludeUrl(url) => {
                write!(f, "Invalid include url {:}", &url)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl From<DotenvError> for DevrcError {
    fn from(error: DotenvError) -> DevrcError {
        DevrcError::Dotenv(error)
    }
}

impl From<tera::Error> for DevrcError {
    fn from(error: tera::Error) -> DevrcError {
        DevrcError::RenderError(error)
    }
}

impl From<IoError> for DevrcError {
    fn from(error: IoError) -> DevrcError {
        DevrcError::IoError(error)
    }
}

impl StdError for DevrcError {}

impl From<anyhow::Error> for DevrcError {
    fn from(error: anyhow::Error) -> Self {
        DevrcError::AnyhowError(error)
    }
}

impl From<DevrcPluginError> for DevrcError {
    fn from(value: DevrcPluginError) -> Self {
        DevrcError::PluginError(value)
    }
}

impl From<StripPrefixError> for DevrcError {
    fn from(_: StripPrefixError) -> Self {
        DevrcError::RuntimeError
    }
}
