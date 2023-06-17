use std::time::Duration;

use devrc_core::{logging::LogLevel, workshop::Designer};

#[derive(Debug, Clone)]
pub struct LoadingConfig {
    pub(crate) level: u32,

    #[allow(dead_code)]
    pub(crate) log_level: LogLevel,

    pub designer: Designer,

    pub cache_ttl: Option<Duration>,
}

impl Default for LoadingConfig {
    fn default() -> Self {
        Self {
            level: 1,
            log_level: Default::default(),
            designer: Designer::default(),
            cache_ttl: None,
        }
    }
}

impl LoadingConfig {
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            level: 1,
            log_level,
            designer: Designer::default(),
            cache_ttl: None,
        }
    }

    pub fn with_level(self, level: u32) -> Self {
        Self { level, ..self }
    }
    pub fn with_log_level(self, log_level: LogLevel) -> Self {
        Self { log_level, ..self }
    }

    pub fn child(self) -> Self {
        Self {
            level: self.level + 1,
            ..self
        }
    }
    pub fn with_cache_ttl(self, ttl: Option<Duration>) -> Self {
        Self {
            cache_ttl: ttl,
            ..self
        }
    }
}
