use crate::{
    config::Config, environment::RawEnvironment, errors::DevrcResult, interpreter::InterpreterKind,
    scope::Scope, variables::RawVariables, workshop::Designer,
};

use serde::Deserialize;

use super::{
    arguments::TaskArguments,
    exec::ExecKind,
    params::{ParamValue, Params},
    result::TaskResult,
};

use crate::evaluate::Evaluatable;

#[derive(Debug, Deserialize, Clone)]
pub struct ComplexCommand {
    name: Option<String>,

    #[serde(default)]
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

    pub fn perform(
        &self,
        _name: &str,
        parent_scope: &Scope,
        args: &TaskArguments,
        config: &Config,
        designer: &Designer,
    ) -> DevrcResult<TaskResult> {
        let mut scope = self.get_scope(parent_scope, args)?;

        let interpreter = self.get_interpreter(config);

        // TODO: register output as variable
        self.exec
            .execute(&mut scope, config, &interpreter, designer)?;

        Ok(TaskResult::new())
    }

    /// Prepare template scope
    pub fn get_scope(&self, parent_scope: &Scope, args: &TaskArguments) -> DevrcResult<Scope> {
        let mut scope = parent_scope.clone();

        // TODO: here devrc can ask user input
        for (key, (value, _)) in args {
            scope.insert_var(key, &value.evaluate(key, &scope)?);
        }

        for (key, value) in &self.variables.evaluate(&scope)? {
            scope.insert_var(key, value);
        }

        match &self.environment.evaluate(&scope) {
            Ok(value) => {
                for (name, value) in value {
                    scope.insert_env(name, value);
                }
            }
            Err(_error) => {}
        }
        // for (name, value) in self.environment.evaluate(&scope) {
        //     scope.insert_env(name, value);
        // }
        // let variables = self.variables.evaluate(parent_scope)?;
        // dbg!(&variables);
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
        }
    }
}
