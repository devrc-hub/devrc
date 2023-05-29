use crate::workshop::Designer;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LogLevel {
    #[default]
    Off = 0, // Quiet mode
    Error = 1,
    Info = 2,  // Show tasks commands
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
