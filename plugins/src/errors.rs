use std::io::Error as IoError;

use crate::options::PluginOption;

pub type DevrcPluginResult<T> = Result<T, DevrcPluginError>;

#[derive(Debug)]
pub enum DevrcPluginError {
    NotFound(String),
    LoadingError(libloading::Error),
    Code { code: i32 },
    Signal,
    IoError(IoError),
    AnyhowError(anyhow::Error),
    InvalidOption(String, PluginOption),
}

impl From<libloading::Error> for DevrcPluginError {
    fn from(value: libloading::Error) -> Self {
        DevrcPluginError::LoadingError(value)
    }
}

impl From<anyhow::Error> for DevrcPluginError {
    fn from(error: anyhow::Error) -> Self {
        DevrcPluginError::AnyhowError(error)
    }
}
