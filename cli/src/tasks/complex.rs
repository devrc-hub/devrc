use crate::{
    config::Config,
    environment::RawEnvironment,
    errors::DevrcResult,
    evaluate::Evaluatable,
    interpreter::{shebang::ShebangDetector, InterpreterKind},
    scope::{child_scope, Scope},
    variables::RawVariables,
};
use std::{cell::RefCell, rc::Rc};

use devrc_core::workshop::Designer;
use serde::Deserialize;

use super::{
    arguments::TaskArguments,
    exec::ExecKind,
    params::{ParamValue, Params},
    result::TaskResult,
    subtask_call::SubtaskCall,
};

use devrc_plugins::execution::ExecutionPluginManager;

#[derive(Debug, Deserialize, Clone)]
pub struct ComplexCommand {
    name: Option<String>,

    #[serde(default, alias = "run")]
    pub exec: ExecKind,

    //shell: ShellVariants,
    pub desc: Option<String>,

    // pub example: Option<Examples>,
    pub example: Option<String>,

    #[serde(default)]
    variables: RawVariables,

    #[serde(default)]
    environment: RawEnvironment<String>,

    #[serde(default)]
    params: Params,

    #[serde(default)]
    pub deps: Vec<String>,

    // #[serde(deserialize_with = "deserialize_interpreter")]
    #[serde(alias = "shell")]
    interpreter: Option<InterpreterKind>,

    #[serde(default)]
    pub subtasks: Vec<SubtaskCall>,
}

impl ComplexCommand {
    pub fn setup_name(&mut self, value: &str) -> DevrcResult<()> {
        self.name = Some(value.to_owned());

        Ok(())
    }

    pub fn setup_params(&mut self, params: Params) -> DevrcResult<()> {
        self.params.merge(params)
    }

    pub fn format_help(&self) -> &str {
        if let Some(value) = &self.desc {
            value
        } else {
            ""
        }
    }

    pub fn format_parameters_help(&self, designer: &Designer) -> DevrcResult<String> {
        self.params.format_help_string(designer)
    }

    pub fn get_interpreter(&self, config: &Config) -> InterpreterKind {
        if let Some(value) = &self.interpreter {
            value.clone()
        } else {
            config.interpreter.clone()
        }
    }

    pub fn perform_code(
        &self,
        interpreter: &InterpreterKind,
        code: &str,
        local_scope: &Scope,
        execution_plugins_registry: Rc<RefCell<ExecutionPluginManager>>,
        config: &Config,
        designer: &Designer,
    ) -> DevrcResult<()> {
        config.log_level.info(code, &designer.command());

        if !config.dry_run {
            if let Some(interpreter) = code.get_interpreter_from_shebang() {
                // Execute script using given shebang
                interpreter.execute_script(code, local_scope, config)?;
            } else {
                // Execute command or complex script
                interpreter.execute(code, local_scope, config, execution_plugins_registry)?;
            }
        }
        Ok(())
    }

    pub fn perform(
        &self,
        _name: &str,
        execution_plugins_registry: Rc<RefCell<ExecutionPluginManager>>,
        parent_scope: Rc<RefCell<Scope>>,
        args: &TaskArguments,
        config: &Config,
        designer: &Designer,
    ) -> DevrcResult<TaskResult> {
        let local_scope = self.compute_execution_scope(parent_scope, args)?;
        let interpreter = self.get_interpreter(config);

        match &self.exec {
            ExecKind::Empty => {}
            ExecKind::String(value) => {
                let code = value.evaluate("exec", &local_scope)?;

                self.perform_code(
                    &interpreter,
                    &code,
                    &local_scope,
                    execution_plugins_registry,
                    config,
                    designer,
                )?;
            }
            ExecKind::List(value) => {
                for (i, item) in value.iter().enumerate() {
                    let code = item.evaluate(&format!("multi_exec_{:}", i), &local_scope)?;

                    self.perform_code(
                        &interpreter,
                        &code,
                        &local_scope,
                        Rc::clone(&execution_plugins_registry),
                        config,
                        designer,
                    )?;
                }
            }
        }

        Ok(TaskResult::new())
    }

    pub fn compute_execution_scope(
        &self,
        scope_ref: Rc<RefCell<Scope>>,
        _args: &TaskArguments,
    ) -> DevrcResult<Scope> {
        let binding = (*scope_ref).borrow();
        binding.compute_execution_scope()
    }

    pub fn get_scope(
        &self,
        parent_scope: Rc<RefCell<Scope>>,
        args: &TaskArguments,
    ) -> DevrcResult<Scope> {
        let mut scope = child_scope(parent_scope, self.name.clone().unwrap_or_default().as_ref());

        // TODO: here devrc can ask user input
        for (key, (value, _)) in args {
            scope.process_binding(key, value)?;
        }

        scope.process_raw_vars(&self.variables)?;
        scope.process_raw_env_vars(&self.environment)?;

        Ok(scope)
    }

    pub fn get_parameters(
        &self,
        _parts: &[String],
    ) -> DevrcResult<indexmap::IndexMap<String, ParamValue>> {
        Ok(self.params.params.clone())
    }

    pub fn has_parameters(&self) -> bool {
        !self.params.params.is_empty()
    }
}

impl<T> From<T> for ComplexCommand
where
    T: ToString,
{
    fn from(v: T) -> ComplexCommand {
        ComplexCommand {
            name: None,
            exec: ExecKind::String(v.to_string()),
            desc: None,
            example: None,
            variables: RawVariables::default(),
            environment: RawEnvironment::default(),
            params: Params::default(),
            deps: Vec::new(),
            interpreter: None,
            subtasks: Vec::new(),
        }
    }
}

impl From<ExecKind> for ComplexCommand {
    fn from(item: ExecKind) -> Self {
        ComplexCommand {
            name: None,
            exec: item,
            desc: None,
            example: None,
            variables: RawVariables::default(),
            environment: RawEnvironment::default(),
            params: Params::default(),
            deps: Vec::new(),
            interpreter: None,
            subtasks: Vec::new(),
        }
    }
}

impl Default for ComplexCommand {
    fn default() -> Self {
        ComplexCommand {
            name: None,
            exec: ExecKind::Empty,
            desc: None,
            example: None,
            variables: RawVariables::default(),
            environment: RawEnvironment::default(),
            params: Params::default(),
            deps: Vec::new(),
            interpreter: None,
            subtasks: Vec::new(),
        }
    }
}
