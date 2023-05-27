use crate::{de::deserialize_some, devrc_log::LogLevel, interpreter::InterpreterKind};
use std::{env, fmt::Debug, path::PathBuf};

use serde::Deserialize;

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
}

#[derive(Debug, Clone)]
pub struct Config {
    pub current_dir: Option<PathBuf>,
    pub interpreter: InterpreterKind,
    pub log_level: LogLevel,
    pub dry_run: bool,
    pub default: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            current_dir: env::current_dir().ok(),
            dry_run: false,
            interpreter: InterpreterKind::default(),
            log_level: LogLevel::Info,
            default: vec![],
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct ExecOptions {
//     pub current_dir: Option<PathBuf>,
//     pub dry_run: bool,
//     pub interpreter: Option<Interpreter>,
//     pub log_level: Option<LogLevel>,
// }

// impl Default for ExecOptions {
//     fn default() -> Self {
//         ExecOptions {
//             current_dir: env::current_dir().ok(),
//             dry_run: false,
//             interpreter: Some(Interpreter::default()),
//             log_level: Some(LogLevel::Info),
//         }
//     }
// }
