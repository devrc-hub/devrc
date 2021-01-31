use std::cmp;

use crate::{config::Config, config::{RawConfig}, environment::{Environment, RawEnvironment}, errors::DevrcResult, raw_devrcfile::RawDevrcfile, scope::Scope, tasks::{Task, Tasks}, variables::{RawVariables, Variables}};

use unicode_width::UnicodeWidthStr;

use indexmap::IndexMap;


#[derive(Debug, Clone, Default)]
pub struct Devrcfile {

    environment: Environment<String>,

    variables: Variables<String>,

    after_script: Option<Task>,
    before_script: Option<Task>,

    before_task: Option<Task>,
    after_task: Option<Task>,
    pub config: Config,

    // TODO: evaluate scope only then add devrcfile
    // pub scope: Scope,

    pub tasks: Tasks,

    pub max_taskname_width: u32
}


impl Devrcfile {

    pub fn add_task(&mut self, name: String, task: Task) -> DevrcResult<()>{
        // println!("Add task: {:?}", new);

        self.tasks.add_task(name, task);

        Ok(())
    }

    pub fn add_before_task(&mut self, new: Option<Task>) -> DevrcResult<()> {
        self.before_task = new;
        Ok(())
    }

    pub fn add_after_task(&mut self, new: Option<Task>) -> DevrcResult<()> {
        self.after_task = new;
        Ok(())
    }

    pub fn add_after_script(&mut self, new: Option<Task>) -> DevrcResult<()> {
        self.after_script = new;
        Ok(())
    }

    pub fn add_before_script(&mut self, new: Option<Task>) -> DevrcResult<()> {
        self.before_script = new;
        Ok(())
    }

    pub fn add_env(&mut self, name: &str, value: &str) -> DevrcResult<()> {
        self.environment.insert(name.to_string(), value.to_string());
        // self.environment.insert_env(&name.to_string(), &value.to_string());
        Ok(())
    }

    pub fn add_var(&mut self, name: &str, value: &str) -> DevrcResult<()> {
        self.variables.insert(name.to_string(), value.to_string());
        // self.scope.insert_var(&name.to_string(), &value.to_string());
        Ok(())
    }

    pub fn add_config(&mut self, config: RawConfig) -> DevrcResult<()> {

        if let Some(Some(value)) = config.interpreter {
            self.config.interpreter = value;
        };

        if let Some(Some(log_level)) = config.log_level {
            self.config.log_level = log_level;
        }

        if let Some(dry_run) = config.dry_run {
            match dry_run {
                Some(dry_run) => {
                    self.config.dry_run = dry_run;
                },
                None => {
                    self.config.dry_run = false;
                }
            }

        }

        Ok(())
    }

    pub fn add_variables(&mut self, variables: RawVariables) -> DevrcResult<()> {
        let scope = self.get_scope();

        if let Ok(value) = &scope {

            match variables.evaluate(value) {
                Ok(value) => {
                    for (name, value) in &value {
                        self.add_var(name, value);
                    }
                },
                Err(error) => return Err(error)
            };
        }
        Ok(())
    }

    pub fn add_env_variables(&mut self, variables: RawEnvironment<String>) -> DevrcResult<()> {
        let scope = self.get_scope();

        if let Ok(value) = &scope {

            match variables.evaluate(value) {
                Ok(value) => {
                    for (name, value) in &value {
                        self.add_env(name, value);
                    }
                },
                Err(error) => return Err(error)
            };
        }

        Ok(())
    }

    /// Add objects from given `RawDevrcfile` to current object
    ///
    /// this method implement merge stategy
    pub fn add_raw_devrcfile(&mut self, file: RawDevrcfile) -> DevrcResult<()>{

        self.add_config(file.config);

        // Field value present or null
        if let Some(value) = file.after_script {
            self.add_after_script(value);
        }

        if let Some(value) = file.before_script {
            self.add_before_script(value);
        }

        if let Some(value) = file.before_task {
            self.add_before_task(value);
        }

        if let Some(value) = file.after_task {
            self.add_after_task(value);
        }

        for (name, task) in file.tasks.items {
            self.add_task(name, task);
        }

        self.add_variables(file.variables);

        self.add_env_variables(file.environment);

        Ok(())
    }

    pub fn get_scope(&self) -> DevrcResult<Scope> {
        let mut scope = Scope::default();

        for (name, value) in &self.variables {
            scope.insert_var(name, value);
        }

        for (name, value) in &self.environment {
            scope.insert_env(name, value);
        }
        Ok(scope)
    }

    pub fn get_max_taskname_width(&self) -> (usize, usize) {

        let mut name_width = 0;
        let mut doc_width = 0;
        for (name, _task) in self.tasks.items.iter(){
            name_width = cmp::max(name_width, UnicodeWidthStr::width(format!("{}", &name).as_str()));
        }
        (name_width, doc_width)
    }

    pub fn find_task(&self, name: &str) -> DevrcResult<Task> {
        self.tasks.find_task(name)
    }

    pub fn run(&mut self, params: &[String]) -> DevrcResult<()>{
        let mut i = 0;

        while i < params.len(){
            let task = self.find_task(&params[i])?;

            let scope = self.get_scope()?;
            task.perform(&params[i], &scope, &params, &self.config)?;
            i += 1;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::{tasks::complex::ComplexCommand, variables::ValueKind, tasks::exec::ExecKind};
    use crate::tasks::*;
    use super::*;

    #[test]
    fn test_variables() {
        let mut raw_variables_1 = RawVariables::default();

        raw_variables_1.add("var1", ValueKind::String("value1".to_owned()));
        raw_variables_1.add("var2", ValueKind::String("value2 {{ var1 }}".to_owned()));

        let mut devrcfile = Devrcfile::default();

        devrcfile.add_variables(raw_variables_1).unwrap();

        let mut raw_variables_2 = RawVariables::default();
        raw_variables_2.add("var3", ValueKind::String("value3 {{ var2 }}".to_owned()));

        devrcfile.add_variables(raw_variables_2).unwrap();

        let mut variables: Variables<String> = Variables::default();

        variables.insert("var1".to_string(), "value1".to_string());
        variables.insert("var2".to_string(), "value2 value1".to_string());
        variables.insert("var3".to_string(),  "value3 value2 value1".to_string());

        assert_eq!(variables, devrcfile.variables);

    }

    #[test]
    fn test_env_variables() {
        let mut raw_variables_1 = RawVariables::default();

        raw_variables_1.add("var1", ValueKind::String("value1".to_owned()));
        raw_variables_1.add("var2", ValueKind::String("value2 {{ var1 }}".to_owned()));

        let mut devrcfile = Devrcfile::default();

        devrcfile.add_variables(raw_variables_1).unwrap();

        let mut raw_environment = RawEnvironment::default();
        raw_environment.add("env_var1", "value3 {{ var2 }}".to_owned());

        devrcfile.add_env_variables(raw_environment).unwrap();

        let mut env: Environment<String> = Environment::default();

        env.insert("env_var1".to_string(), "value3 value2 value1".to_string());

        assert_eq!(env, devrcfile.environment);

    }

    #[test]
    fn test_get_scope() {

        let mut raw_variables_1 = RawVariables::default();

        raw_variables_1.add("var1", ValueKind::String("value1".to_owned()));
        raw_variables_1.add("var2", ValueKind::String("value2 {{ var1 }}".to_owned()));

        let mut devrcfile = Devrcfile::default();

        devrcfile.add_variables(raw_variables_1).unwrap();

        let mut raw_environment = RawEnvironment::default();
        raw_environment.add("env_var1", "value3 {{ var2 }}".to_owned());

        devrcfile.add_env_variables(raw_environment).unwrap();

        let mut scope = Scope::default();

        // let scope = Scope { variables: {"var2": "value2 value1", "var1": "value1"}, environment: {"env_var1": "value3 value2 value1"} }
        scope.insert_var("var1",  "value1");
        scope.insert_var("var2", "value2 value1");

        scope.insert_env("env_var1", "value3 value2 value1");

        assert_eq!(scope, devrcfile.get_scope().unwrap());
    }

    #[test]
    fn test_find_task() {

        let mut devrcfile = Devrcfile::default();

        let task = Task::default();
        devrcfile.add_task("task_1".to_owned(), task).unwrap();

        for i in 1..10 {
            let mut cmd = ComplexCommand::default();
            cmd.exec = ExecKind::String(format!("echo \"Hello {:}\"", i));

            let task = Task::ComplexCommand(cmd);

            devrcfile.add_task(format!("task_{:}", i), task).unwrap();
        }

        match devrcfile.find_task("task_3").unwrap() {
            TaskKind::ComplexCommand(
                ComplexCommand { exec: ExecKind::String(exec), .. }) => {
                assert_eq!(exec, "echo \"Hello 3\"");

            },
            _ => {
                assert!(false);
            }
        }
    }
}
