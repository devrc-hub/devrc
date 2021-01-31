use std::{env, fmt::Debug, path::PathBuf};
use crate::{de::{deserialize_some}, interpreter::{DEFAULT_SHELL, get_default_shell,  Interpreter}};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum DefaultOption {
    Empty,
    String(String),
    List(Vec<String>),
}

impl Default for DefaultOption {
    fn default() -> Self {
        DefaultOption::Empty
    }
}


#[derive(Debug, Deserialize, Clone)]
pub enum LogLevel {
    Warn,
    Debug,
    Error,
    Info,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

fn default_global() -> bool{
    false
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RawConfig {

    #[serde(default, deserialize_with = "deserialize_some")]
    pub default: Option<DefaultOption>,

    // #[serde(default = "default_shell")]
    #[serde(default, deserialize_with = "deserialize_some", rename="shell")]
    pub interpreter: Option<Option<Interpreter>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub log_level: Option<Option<LogLevel>>,

    // #[serde(default = "default_global")]
    pub global: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub dry_run: Option<Option<bool>>
}


#[derive(Debug, Clone)]
pub struct Config {
    pub current_dir: Option<PathBuf>,
    pub interpreter: Interpreter,
    pub log_level: LogLevel,
    pub dry_run: bool
}

impl Default for Config {
    fn default() -> Self {
        Config {
            current_dir: env::current_dir().ok(),
            dry_run: false,
            interpreter: Interpreter::default(),
            log_level: LogLevel::Error
        }
    }
}


#[derive(Debug, Clone)]
pub struct ExecOptions {

    pub current_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub interpreter: Option<Interpreter>,
    pub log_level: Option<LogLevel>
}

impl Default for ExecOptions {
    fn default() -> Self {

        ExecOptions {
            current_dir: env::current_dir().ok(),
            dry_run: false,
            interpreter: Some(Interpreter::default()),
            log_level: Some(LogLevel::Error)

        }
    }
}
