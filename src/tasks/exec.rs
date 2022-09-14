use serde::Deserialize;

use crate::{
    config::Config,
    errors::DevrcResult,
    evaluate::Evaluatable,
    interpreter::{InterpreterKind, ShebangDetector},
    scope::Scope,
    workshop::Designer,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExecKind {
    Empty,
    String(String),
    // Complex(indexmap::IndexMap<String, String>),
    List(Vec<String>),
}

impl ExecKind {
    pub fn execute(
        &self,
        scope: &mut Scope,
        config: &Config,
        interpreter: &InterpreterKind,
        designer: &Designer,
    ) -> DevrcResult<i32> {
        match self {
            ExecKind::Empty => {
                // Nothing to do
                Ok(0)
            }
            ExecKind::String(value) => {
                let cmd = value.evaluate("exec", scope)?;

                config.log_level.info(&cmd, &designer.command());

                if !config.dry_run {
                    if let Some(interpreter) = cmd.get_interpreter_from_shebang() {
                        // Execute script using given shebang
                        interpreter.execute_script(&cmd, scope, config)
                    } else {
                        // Execute command or complex script
                        interpreter.execute(&cmd, scope, config)
                    }
                } else {
                    Ok(0)
                }
            }
            ExecKind::List(value) => {
                for (i, item) in value.iter().enumerate() {
                    let cmd = item.evaluate(&format!("multi_exec_{:}", i), scope)?;

                    config.log_level.info(&cmd, &designer.command());

                    if !config.dry_run {
                        if let Some(interpreter) = cmd.get_interpreter_from_shebang() {
                            // Execute script using given shebang
                            interpreter.execute_script(&cmd, scope, config)?;
                        } else {
                            // Execute command or complex script
                            interpreter.execute(&cmd, scope, config)?;
                        }
                    }
                }
                Ok(0)
            }
        }
    }
}

impl Default for ExecKind {
    fn default() -> Self {
        Self::Empty
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_name() {}
}
