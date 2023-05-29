pub mod plugins;
pub mod shebang;
pub mod system;

use crate::{
    config::Config,
    errors::{DevrcError, DevrcResult},
    scope::Scope,
};
use std::{
    convert::{TryFrom, TryInto},
    env,
    fmt::{self, Display},
};

use serde::Deserialize;
use std::{cell::RefCell, rc::Rc};

use self::{plugins::PluginInterpreter, system::SystemShell};

use devrc_plugins::{config::ExecutionConfig, execution::ExecutionPluginManager};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum InterpreterKind {
    PluginInterpreter(PluginInterpreter),
    Internal(SystemShell),
}

impl InterpreterKind {
    pub fn execute(
        &self,
        code: &str,
        scope: &Scope,
        config: &Config,
        registry: Rc<RefCell<ExecutionPluginManager>>,
    ) -> DevrcResult<i32> {
        match self {
            InterpreterKind::Internal(internal_shell) => {
                internal_shell.execute(code, scope, config)
            }
            InterpreterKind::PluginInterpreter(interpreter) => {
                let mut manager = (*registry)
                    .try_borrow_mut()
                    .map_err(|_| DevrcError::RuntimeError)?;
                let plugin = manager.get_plugin(&interpreter.runtime)?;
                let options = interpreter
                    .try_into()
                    .map_err(|_| DevrcError::RuntimeError)?;
                Ok(plugin.execute(options, code, &scope.environment)?)
            }
        }
    }
}

impl Default for InterpreterKind {
    fn default() -> Self {
        InterpreterKind::Internal(SystemShell::default())
    }
}

impl Display for InterpreterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterKind::Internal(interpreter) => {
                write!(f, "{}", interpreter)
            }
            InterpreterKind::PluginInterpreter(interpreter) => {
                write!(f, "{}", interpreter)
            }
        }
    }
}

impl TryFrom<&PluginInterpreter> for ExecutionConfig {
    type Error = DevrcError;

    fn try_from(value: &PluginInterpreter) -> Result<Self, Self::Error> {
        Ok(Self {
            runtime: value.runtime.clone(),
            current_dir: env::current_dir().ok(),
            args: value.args.clone().unwrap_or_default(),
            options: value
                .options
                .clone()
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {}
}
