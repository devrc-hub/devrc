use crate::interpreter::InterpreterKind;
use std::{env, fmt::Debug, path::PathBuf};

use devrc_core::logging::LogLevel;

#[derive(Debug, Clone)]
pub struct Config {
    pub current_dir: Option<PathBuf>,
    pub interpreter: InterpreterKind,
    pub log_level: LogLevel,
    pub dry_run: bool,
    pub default: Vec<String>,
    pub plugins: indexmap::IndexMap<String, PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            current_dir: env::current_dir().ok(),
            dry_run: false,
            interpreter: InterpreterKind::default(),
            log_level: LogLevel::Info,
            default: vec![],
            plugins: indexmap::IndexMap::new(),
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
