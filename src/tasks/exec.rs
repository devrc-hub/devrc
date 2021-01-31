use serde::{Deserialize, Deserializer};

use crate::{config::Config, errors::{DevrcError, DevrcResult}, evaluate::Evaluatable, interpreter::{Interpreter, ShebangDetector}, scope::Scope};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExecKind {
    Empty,
    String(String),
    // Complex(indexmap::IndexMap<String, String>),
    List(Vec<String>),
}


impl ExecKind {

    pub fn execute(&self, scope: &mut Scope, config: &Config, interpreter: &Interpreter) -> DevrcResult<()>{

        match self {
            ExecKind::Empty => {
                return Err(DevrcError::NotImplemented)
            },
            ExecKind::String(value) => {
                let cmd = value.evaluate("exec", scope)?;

                if !config.dry_run {

                    if let Some(interpreter) = cmd.get_interpreter_from_shebang() {
                        // Execute script using given shebang
                        interpreter.execute_script(&cmd, scope, config)?
                    } else {
                        // Execute command or complex script
                        interpreter.execute(&cmd, scope, config)?
                    }
                }
            },
            ExecKind::List(value) => {

                for (i, item) in value.iter().enumerate() {
                    let cmd =  item.evaluate(&format!("multi_exec_{:}", i), &scope)?;

                    if !config.dry_run {
                        if let Some(interpreter) = cmd.get_interpreter_from_shebang() {
                            // Execute script using given shebang
                            interpreter.execute_script(&cmd, scope, config)?;

                        } else {
                            // Execute command or complex script
                            interpreter.execute(&cmd, scope, config)?;

                        }
                    }

                    // match item.evaluate("multi_exec", &scope) {
                    //     Ok(value) => {
                    //          cmd.execute(&scope)?
                    //     println!("{:?}", value)
                    // }
                    // Err(error) => {
                    //     return Err(error)
                    // }
                    // }
                }
            }
        };
        Ok(())
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {

    }
}
