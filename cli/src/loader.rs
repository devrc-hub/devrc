use crate::{devrc_log::LogLevel, workshop::Designer};

#[derive(Debug, Clone)]
pub struct LoadingConfig {
    pub(crate) level: u32,

    #[allow(dead_code)]
    pub(crate) log_level: LogLevel,

    pub designer: Designer,
}

impl Default for LoadingConfig {
    fn default() -> Self {
        Self {
            level: 1,
            log_level: Default::default(),
            designer: Designer::default(),
        }
    }
}

impl LoadingConfig {
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            level: 1,
            log_level,
            designer: Designer::default(),
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
}
