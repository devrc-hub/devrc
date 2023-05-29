use super::system::SystemShell;

pub trait ShebangDetector {
    fn get_interpreter_from_shebang(&self) -> Option<SystemShell>;
}

impl ShebangDetector for String {
    fn get_interpreter_from_shebang(&self) -> Option<SystemShell> {
        let first_line = self.lines().next().unwrap_or("");

        if !first_line.starts_with("#!") {
            return None;
        }

        let mut parts = first_line[2..].splitn(2, |c| c == ' ' || c == '\t');

        if let Some(value) = parts.next() {
            let mut args = Vec::new();

            if let Some(value) = parts.next().map(|arg| arg.to_owned()) {
                args.push(value)
            };

            Some(SystemShell {
                interpreter: value.to_owned(),
                args,
            })
        } else {
            None
        }
    }
}

impl ShebangDetector for &str {
    fn get_interpreter_from_shebang(&self) -> Option<SystemShell> {
        let first_line = self.lines().next().unwrap_or("");

        if !first_line.starts_with("#!") {
            return None;
        }

        let mut parts = first_line[2..].splitn(2, |c| c == ' ' || c == '\t');

        if let Some(value) = parts.next() {
            let mut args = Vec::new();

            if let Some(value) = parts.next().map(|arg| arg.to_owned()) {
                args.push(value)
            };

            Some(SystemShell {
                interpreter: value.to_owned(),
                args,
            })
        } else {
            None
        }
    }
}
