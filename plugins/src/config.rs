use std::path::PathBuf;

use devrc_core::{logging::LogLevel, workshop::Designer};

use crate::options::PluginOption;

#[derive(Default, Debug)]
pub struct Config {
    pub designer: Designer,
    pub logger: LogLevel,
}

#[derive(Default, Debug)]
pub struct ExecutionConfig {
    pub runtime: String,
    pub current_dir: Option<PathBuf>,
    pub args: Vec<String>,
    pub options: indexmap::IndexMap<String, PluginOption>,
}
