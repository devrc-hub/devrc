use serde::Deserialize;

use crate::workshop::Designer;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    #[serde(rename = "off")]
    Off = 0, // Quiet mode

    #[serde(rename = "error")]
    Error = 1,

    #[serde(rename = "info")]
    Info = 2, // Show tasks commands

    #[serde(rename = "debug")]
    Debug = 3, // Show info messages such `==> Running task`
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Off
    }
}

impl From<u8> for LogLevel {
    fn from(level: u8) -> Self {
        match level {
            0 => LogLevel::Off,
            1 => LogLevel::Error,
            2 => LogLevel::Info,
            3 => LogLevel::Debug,
            x if x > 3 => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }
}

impl LogLevel {
    pub fn info(&self, content: &str, designer: &Designer) {
        if *self >= Self::Info {
            eprintln!("{}{}{}", designer.prefix(), &content, designer.suffix());
        }
    }

    pub fn debug(&self, content: &str, designer: &Designer) {
        if *self >= Self::Debug {
            eprintln!("{}{}{}", designer.prefix(), &content, designer.suffix());
        }
    }

    pub fn error(&self, content: &str, designer: &Designer) {
        if *self >= Self::Error {
            eprintln!("{}{}{}", designer.prefix(), &content, designer.suffix());
        }
    }
}
