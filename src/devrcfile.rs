use std::{cmp, path::PathBuf};

use crate::{
    config::{Config, DefaultOption, RawConfig},
    devrc_log::LogLevel,
    environment::{EnvFile, Environment, RawEnvironment},
    errors::{DevrcError, DevrcResult},
    raw_devrcfile::RawDevrcfile,
    scope::Scope,
    tasks::{Task, TaskKind, Tasks},
    variables::{RawVariables, Variables},
    workshop::Designer,
};

use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Default)]
pub struct Devrcfile {
    pub environment: Environment<String>,

    pub variables: Variables<String>,

    after_script: Option<Task>,
    before_script: Option<Task>,

    before_task: Option<Task>,
    after_task: Option<Task>,
    pub config: Config,

    // TODO: evaluate scope only then add devrcfile
    // pub scope: Scope,
    pub tasks: Tasks,

    pub max_taskname_width: u32,

    pub designer: Designer,
}

impl Devrcfile {
    pub fn add_task(&mut self, name: String, task: Task) -> DevrcResult<()> {
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

        if let Some(log_level) = config.log_level {
            self.config.log_level = match log_level {
                Some(log_level) => log_level,
                None => LogLevel::Info,
            };
        }

        if let Some(dry_run) = config.dry_run {
            match dry_run {
                Some(dry_run) => {
                    self.config.dry_run = dry_run;
                }
                None => {
                    self.config.dry_run = false;
                }
            }
        }

        if let Some(DefaultOption::List(values)) = config.default {
            self.config.default = values;
        }

        Ok(())
    }

    pub fn setup_dry_run(&mut self, dry_run: bool) -> DevrcResult<()> {
        self.config.dry_run = dry_run;
        Ok(())
    }

    pub fn setup_log_level(&mut self, level: LogLevel) -> DevrcResult<()> {
        self.config.log_level = level;
        Ok(())
    }

    pub fn add_variables(&mut self, variables: RawVariables) -> DevrcResult<()> {
        let scope = self.get_scope();

        if let Ok(value) = &scope {
            match variables.evaluate(value) {
                Ok(value) => {
                    for (name, value) in &value {
                        self.add_var(name, value)?;
                    }
                }
                Err(error) => return Err(error),
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
                        self.add_env(name, value)?;
                    }
                }
                Err(error) => return Err(error),
            };
        }

        Ok(())
    }

    pub fn add_env_file(&mut self, files: EnvFile, base_path: Option<&PathBuf>) -> DevrcResult<()> {
        for (key, value) in files.load(base_path)? {
            self.add_env(&key, &value)?;
        }

        Ok(())
    }

    /// Add objects from given `RawDevrcfile` to current object
    ///
    /// this method implement merge stategy
    pub fn add_raw_devrcfile(&mut self, file: RawDevrcfile) -> DevrcResult<()> {
        self.add_config(file.config)?;

        // Field value present or null
        if let Some(value) = file.after_script {
            self.add_after_script(value)?;
        }

        if let Some(value) = file.before_script {
            self.add_before_script(value)?;
        }

        if let Some(value) = file.before_task {
            self.add_before_task(value)?;
        }

        if let Some(value) = file.after_task {
            self.add_after_task(value)?;
        }

        for (name, task) in file.tasks.items {
            self.add_task(name, task)?;
        }

        if let Some(files) = file.envs_files {
            self.add_env_file(files, file.path.as_ref())?;
        }

        self.add_variables(file.variables)?;

        self.add_env_variables(file.environment)?;

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

    /// Get task doct objects
    // pub fn get_tasks_docs(&self) -> std::iter::Map<indexmap::map::Iter<String, crate::tasks::TaskKind>, |(&String, &crate::tasks::TaskKind)| -> ()> {
    pub fn get_tasks_docs(&self) -> impl Iterator<Item = (&String, &TaskKind)> {
        self.tasks.items.iter().map(|(key, value)| (key, value))
    }

    pub fn get_vars(&self) -> impl Iterator<Item = (&String, &String)> {
        self.variables.iter().map(|(key, value)| (key, value))
    }

    pub fn get_environment_vars(&self) -> impl Iterator<Item = (&String, &String)> {
        self.environment.iter().map(|(key, value)| (key, value))
    }

    pub fn get_max_taskname_width(&self) -> (usize, usize) {
        let mut name_width = 0;
        let doc_width = 0;
        for (name, _task) in self.get_tasks_docs() {
            name_width = cmp::max(
                name_width,
                UnicodeWidthStr::width(name.to_string().as_str()),
            );
        }
        (name_width, doc_width)
    }

    pub fn find_task(&self, name: &str) -> DevrcResult<&Task> {
        match name {
            "before_script" => self.before_script.as_ref().ok_or(DevrcError::TaskNotFound),
            "after_script" => self.after_script.as_ref().ok_or(DevrcError::TaskNotFound),
            "before_task" | "before_task_" => {
                self.before_task.as_ref().ok_or(DevrcError::TaskNotFound)
            }
            "after_task" | "after_task_" => {
                self.after_task.as_ref().ok_or(DevrcError::TaskNotFound)
            }
            _ => Ok(self.tasks.find_task(name)?),
        }
    }

    // Execute hooks if they exists
    pub fn run_hook(&self, name: &str, task_name: Option<&str>) -> DevrcResult<()> {
        let scope = self.get_scope()?;

        if let Ok(task) = self.find_task(name) {
            let hook_display_name = if let Some(task_name) = task_name {
                format!("{}_{}", name, task_name)
            } else {
                name.to_string()
            };

            task.perform(
                &hook_display_name,
                &scope,
                &[],
                &self.config,
                &self.designer,
            )?;

            // self.run_task(&hook_display_name, task, &[])?;
        }

        Ok(())
    }

    pub fn run_task(&self, name: &str, task: &TaskKind, params: &[String]) -> DevrcResult<()> {
        let scope = self.get_scope()?;

        if let Some(deps) = task.get_dependencies() {
            self.config.log_level.debug(
                &format!("\n==> Running task `{}` dependencies: ...", &name),
                &self.designer.banner(),
            );

            for dependency_task_name in deps {
                let dependency_task = self.find_task(dependency_task_name)?;
                self.run_task(dependency_task_name, dependency_task, &[])?;
            }
        }

        self.run_hook("before_task", Some(&name))?;

        task.perform(name, &scope, params, &self.config, &self.designer)?;

        self.run_hook("after_task", Some(&name))?;
        Ok(())
    }

    pub fn run(&self, params: &[String]) -> DevrcResult<()> {
        let mut i = 0;

        let tasks_names = if params.is_empty() {
            self.config.default.clone()
        } else {
            params.to_vec()
        };

        let mut tasks: Vec<(&str, &TaskKind, &[String])> = Vec::new();

        while i < tasks_names.len() {
            let name = &tasks_names[i];

            let task = self.find_task(&tasks_names[i])?;

            tasks.push((name, task, &[]));

            i += 1;
        }

        self.detect_circular_dependencies(
            &tasks
                .iter()
                .map(|x| (x.0, x.1))
                .collect::<Vec<(&str, &TaskKind)>>(),
        )?;

        self.run_hook("before_script", None)?;

        for (name, task, params) in tasks {
            self.run_task(name, task, params)?;
        }

        self.run_hook("after_script", None)?;

        Ok(())
    }

    // Try to prepare some base for circular dependencies checker
    // Checker must run before tasks execution.
    pub fn detect_circular_dependencies(&self, _tasks: &[(&str, &TaskKind)]) -> DevrcResult<bool> {
        // Err(DevrcError::CircularDependencies)
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tasks::{complex::ComplexCommand, exec::ExecKind, *},
        variables::ValueKind,
    };

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
        variables.insert("var3".to_string(), "value3 value2 value1".to_string());

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
        scope.insert_var("var1", "value1");
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
            let cmd = ComplexCommand::from(format!("echo \"Hello {:}\"", i));

            let task = Task::ComplexCommand(cmd);

            devrcfile.add_task(format!("task_{:}", i), task).unwrap();
        }

        match devrcfile.find_task("task_3").unwrap() {
            TaskKind::ComplexCommand(ComplexCommand {
                exec: ExecKind::String(exec),
                ..
            }) => {
                assert_eq!(exec, "echo \"Hello 3\"");
            }
            _ => {
                unreachable!();
            }
        }
    }
}
