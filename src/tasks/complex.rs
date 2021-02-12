use crate::{config::{Config}, environment::RawEnvironment, errors::DevrcResult, evaluate::Evaluatable, interpreter::{Interpreter}, scope::Scope, variables::RawVariables};

use serde::{Deserialize, Deserializer};

use super::{examples::Examples, exec::ExecKind, params::Params};


#[derive(Debug, Deserialize, Clone)]
pub struct ComplexCommand {

    name: Option<String>,

    pub exec: ExecKind,
    //shell: ShellVariants,
    pub desc: Option<String>,

    examples: Option<Examples>,

    #[serde(default)]
    variables: RawVariables,

    #[serde(default)]
    environment: RawEnvironment<String>,

    #[serde(default)]
    params: Params,

    #[serde(default)]
    deps: Vec<String>,

    // #[serde(deserialize_with = "deserialize_interpreter")]
    shell: Option<Interpreter>
}


impl ComplexCommand {
    pub fn format_help(&self) -> &str{
        if let Some(value) = &self.desc {
            value
        } else {
            ""
        }
    }

    pub fn get_interpreter(&self, config: &Config) -> Interpreter {
        if let Some(value) = &self.shell {
            value.clone()
        }
        else {
            config.interpreter.clone()
        }
    }

    pub fn perform(&self, name: &str, parent_scope: &Scope, params: &[String], config: &Config) -> DevrcResult<()>{

        let mut scope = self.get_scope(parent_scope, params)?;

        let interpreter = self.get_interpreter(&config);

        // TODO: register output as variable
        self.exec.execute(&mut scope, config, &interpreter)
    }

    /// Prepare template scope
    pub fn get_scope(&self, parent_scope: &Scope, params: &[String]) -> DevrcResult<Scope>{
        let mut scope = parent_scope.clone();

        for (key, value) in &self.variables.evaluate(&parent_scope)? {

            scope.insert_var(key, value);
        }

        match &self.environment.evaluate(&scope) {
            Ok(value) => {
                 for (name, value) in value {
                        scope.insert_env(name, value);
                    }
            },
            Err(error) =>{ }
        }
        // for (name, value) in self.environment.evaluate(&scope) {
        //     scope.insert_env(name, value);
        // }
        // let variables = self.variables.evaluate(parent_scope)?;
        // dbg!(&variables);
        Ok(scope)
    }
}

impl<T> From<T> for ComplexCommand where T: ToString {

    fn from(v: T) -> ComplexCommand {
        ComplexCommand{
            name: None,
            exec: ExecKind::String(v.to_string()),
            desc: None,
            examples: None,
            variables: RawVariables::default(),
            environment: RawEnvironment::default(),
            params: Params::default(),
            deps: Vec::new(),
            shell: None
        }
    }
}

impl Default for ComplexCommand {
    fn default() -> Self {
        ComplexCommand{
            name: None,
            exec: ExecKind::Empty,
            desc: None,
            examples: None,
            variables: RawVariables::default(),
            environment: RawEnvironment::default(),
            params: Params::default(),
            deps: Vec::new(),
            shell: None
        }

    }
}
