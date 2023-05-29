use crate::{de::deserialize_some, interpreter::InterpreterKind};
use std::{fmt::Debug, path::PathBuf};

use devrc_core::logging;
use serde::Deserialize;
use std::convert::From;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(untagged)]
pub enum DefaultOption {
    #[default]
    Empty,
    String(String),
    List(Vec<String>),
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RawConfig {
    #[serde(default, deserialize_with = "deserialize_some")]
    pub default: Option<DefaultOption>,

    // #[serde(default = "default_shell")]
    #[serde(default, deserialize_with = "deserialize_some", alias = "shell")]
    pub interpreter: Option<Option<InterpreterKind>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub log_level: Option<Option<LogLevel>>,

    // #[serde(default = "default_global")]
    pub global: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub dry_run: Option<Option<bool>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub plugins: Option<indexmap::IndexMap<String, PathBuf>>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LogLevel {
    #[default]
    #[serde(rename = "off")]
    Off = 0, // Quiet mode

    #[serde(rename = "error")]
    Error = 1,

    #[serde(rename = "info")]
    Info = 2, // Show tasks commands

    #[serde(rename = "debug")]
    Debug = 3, // Show info messages such `==> Running task`
}

impl From<u8> for LogLevel {
    fn from(level: u8) -> Self {
        match level {
            0 => LogLevel::Error,
            1 => LogLevel::Info,
            x if x >= 2 => LogLevel::Debug,
            _ => LogLevel::Debug,
        }
    }
}

impl From<LogLevel> for logging::LogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => logging::LogLevel::Off,
            LogLevel::Error => logging::LogLevel::Error,
            LogLevel::Info => logging::LogLevel::Info,
            LogLevel::Debug => logging::LogLevel::Debug,
        }
    }
}
