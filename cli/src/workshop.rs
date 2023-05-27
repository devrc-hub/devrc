use ansi_term::{Color, Prefix, Style, Suffix};

#[derive(Clone, Copy, Debug)]
pub struct Designer {
    style: Style,
}

impl Default for Designer {
    fn default() -> Self {
        Self {
            style: Style::new(),
        }
    }
}

impl Designer {
    // Apply command styles
    pub fn command(&self) -> Self {
        Self {
            style: Style::new().bold(),
        }
    }

    pub fn banner(&self) -> Self {
        Self {
            style: Style::new().fg(Color::Cyan).bold(),
        }
    }

    pub fn message(&self) -> Self {
        Self {
            style: Style::new(),
        }
    }

    pub fn parameter_name(&self) -> Self {
        Self {
            style: Style::new().fg(Color::Green),
        }
    }

    pub fn parameter_value(&self) -> Self {
        Self {
            style: Style::new().fg(Color::Cyan),
        }
    }

    pub fn doc(&self) -> Self {
        Self {
            style: Style::new(),
        }
    }

    pub fn error(&self) -> Self {
        Self {
            style: Style::new(),
        }
    }

    pub fn task_name(&self) -> Self {
        Self {
            style: Style::new().bold(),
        }
    }

    pub fn variable(&self) -> Self {
        self.task_name()
    }

    pub fn evariable(&self) -> Self {
        self.task_name()
    }

    pub fn prefix(&self) -> Prefix {
        self.style.prefix()
    }

    pub fn suffix(&self) -> Suffix {
        self.style.suffix()
    }
}
