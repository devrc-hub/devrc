use std::{cell::RefCell, cmp, rc::Rc};

use crate::{
    config::Config,
    environment::{Environment, RawEnvironment},
    errors::{DevrcError, DevrcResult},
    raw::{
        config::{DefaultOption, RawConfig},
        devrcfile::{Kind, RawDevrcfile},
    },
    scope::{child_scope, Scope},
    tasks::{
        arguments::{extract_task_args, TaskArguments},
        Task, TaskKind, Tasks,
    },
    variables::RawVariables,
};

use devrc_core::workshop::Designer;
use unicode_width::UnicodeWidthStr;

use devrc_plugins::execution::ExecutionPluginManager;

#[derive(Debug, Clone, Default)]
pub struct Devrcfile {
    after_script: Option<Task>,
    before_script: Option<Task>,

    before_task: Option<Task>,
    after_task: Option<Task>,
    pub config: Config,

    pub tasks: Tasks,

    pub max_taskname_width: u32,

    pub designer: Designer,

    pub scope: Rc<RefCell<Scope>>,

    pub execution_plugin_registry: Rc<RefCell<ExecutionPluginManager>>,
}

impl Devrcfile {
    pub fn with_scope(scope: Rc<RefCell<Scope>>) -> Self {
        Devrcfile {
            scope: Rc::clone(&scope),
            ..Default::default()
        }
    }
    pub fn with_execution_plugin_manager(
        self,
        manager: Rc<RefCell<ExecutionPluginManager>>,
    ) -> Self {
        Devrcfile {
            execution_plugin_registry: manager,
            ..self
        }
    }

    pub fn add_task(&mut self, name: String, task: Task) -> DevrcResult<()> {
        self.tasks.add_task(name, task)
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

    pub fn add_config(&mut self, config: RawConfig, kind: &Kind) -> DevrcResult<()> {
        if matches!(
            kind,
            Kind::Directory | Kind::DirectoryLocal | Kind::Args | Kind::Global | Kind::StdIn
        ) {
            if let Some(Some(value)) = config.interpreter {
                self.config.interpreter = value;
            };

            if let Some(log_level) = config.log_level {
                self.config.log_level = match log_level {
                    Some(log_level) => log_level.into(),
                    None => devrc_core::logging::LogLevel::Info,
                };
            }

            if let Some(DefaultOption::List(values)) = config.default {
                self.config.default = values;
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

            if let Some(plugins_paths) = config.plugins {
                for (name, path) in plugins_paths {
                    self.config.plugins.insert(name, path);
                }
            }
        }

        Ok(())
    }

    pub fn setup_dry_run(&mut self, dry_run: bool) -> DevrcResult<()> {
        self.config.dry_run = dry_run;
        Ok(())
    }

    pub fn setup_log_level(&mut self, level: devrc_core::logging::LogLevel) -> DevrcResult<()> {
        self.config.log_level = level;
        Ok(())
    }

    // Add variables to global scope
    pub fn process_variables(&mut self, variables: RawVariables) -> DevrcResult<()> {
        let mut global_scope = (*self.scope)
            .try_borrow_mut()
            .map_err(|_| DevrcError::RuntimeError)?;
        global_scope.process_raw_vars(&variables)
    }

    // Add variables to global scope
    pub fn process_env_variables(&mut self, variables: RawEnvironment<String>) -> DevrcResult<()> {
        let mut global_scope = (*self.scope)
            .try_borrow_mut()
            .map_err(|_| DevrcError::RuntimeError)?;
        global_scope.process_raw_env_vars(&variables)
    }

    pub fn process_env_files_variables(
        &mut self,
        variables: Environment<String>,
    ) -> DevrcResult<()> {
        let mut global_scope = (*self.scope)
            .try_borrow_mut()
            .map_err(|_| DevrcError::RuntimeError)?;
        global_scope.process_rendered_env_vars(&variables)
    }

    // pub fn add_env_file(&mut self, files: EnvFile, base_path: Option<&PathBuf>) -> DevrcResult<()> {
    //     for (key, value) in files.load(base_path)? {
    //         let mut global_scope = (&*self.scope)
    //             .try_borrow_mut()
    //             .map_err(|_| DevrcError::RuntimeError)?;
    //         global_scope.insert_env(&key.to_string(), &value.to_owned());
    //     }

    //     Ok(())
    // }

    pub fn get_scope_copy(&self) -> DevrcResult<Scope> {
        Ok(((*self.scope)
            .try_borrow()
            .map_err(|_| DevrcError::RuntimeError)?)
        .clone())
    }

    /// Add objects from given `RawDevrcfile` to current object
    ///
    /// this method implement merge stategy
    pub fn add_raw_devrcfile(&mut self, file: RawDevrcfile, kind: &Kind) -> DevrcResult<()> {
        self.add_config(file.config, kind)?;

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

        self.process_env_files_variables(file.files_environment)?;

        self.process_variables(file.variables)?;

        self.process_env_variables(file.environment)?;

        Ok(())
    }

    /// Get task doct objects
    // pub fn get_tasks_docs(&self) -> std::iter::Map<indexmap::map::Iter<String, crate::tasks::TaskKind>, |(&String, &crate::tasks::TaskKind)| -> ()> {
    pub fn get_tasks_docs(&self) -> impl Iterator<Item = (&String, &TaskKind)> {
        self.tasks.items.iter().map(|(key, value)| (key, value))
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
        if let Ok(task) = self.find_task(name) {
            let hook_display_name = if let Some(task_name) = task_name {
                format!("{}_{}", name, task_name)
            } else {
                name.to_string()
            };

            task.perform(
                &hook_display_name,
                Rc::clone(&self.execution_plugin_registry),
                Rc::clone(&self.scope),
                &TaskArguments::new(),
                &self.config,
                &self.designer,
            )?;
            // self.run_task(&hook_display_name, task, &[])?;
        }

        Ok(())
    }

    pub fn run_task(
        &self,
        name: &str,
        task: &TaskKind,
        args: TaskArguments,
        parent_scope: Rc<RefCell<Scope>>,
    ) -> DevrcResult<()> {
        // Execute dependencies tasks
        if let Some(deps) = task.get_dependencies() {
            self.config.log_level.debug(
                &format!("\n==> Running task `{}` dependencies: ...", &name),
                &self.designer.banner(),
            );

            for dependency_task_name in deps {
                let dependency_task = self.find_task(dependency_task_name)?;
                self.run_task(
                    dependency_task_name,
                    dependency_task,
                    args.clone(),
                    Rc::clone(&parent_scope),
                )?;
            }
        }

        // Execute subtasks before main task
        let scope = Rc::new(RefCell::new(task.get_scope(
            name,
            Rc::clone(&parent_scope),
            &args,
        )?));

        if let Some(subtasks) = task.get_subtasks() {
            self.config.log_level.debug(
                &format!("\n==> Running subtasks `{}`: ...", &name),
                &self.designer.banner(),
            );

            for subtask_call in subtasks {
                let subtask = self.find_task(&subtask_call.name)?;

                let mut subtask_scope = child_scope(
                    Rc::clone(&scope),
                    &format!("\"{:}\" subtasks scope", &subtask_call.name),
                );

                subtask_scope.process_raw_vars(&subtask_call.variables)?;
                subtask_scope.process_raw_env_vars(&subtask_call.environment)?;

                self.run_task(
                    &subtask_call.name,
                    subtask,
                    args.clone(),
                    Rc::new(RefCell::new(subtask_scope)),
                )?;
            }
        }

        self.run_hook("before_task", Some(name))?;

        let _ = task.perform(
            name,
            Rc::clone(&self.execution_plugin_registry),
            Rc::clone(&scope),
            &args,
            &self.config,
            &self.designer,
        )?;

        self.run_hook("after_task", Some(name))?;
        Ok(())
    }

    pub fn run(&self, params: &[String]) -> DevrcResult<()> {
        let mut i = 0;

        let tasks_names = if params.is_empty() {
            self.config.default.clone()
        } else {
            params.to_vec()
        };

        let mut tasks: Vec<(&str, &TaskKind, TaskArguments)> = Vec::new();

        while i < tasks_names.len() {
            let name = &tasks_names[i];

            let task = self.find_task(&tasks_names[i])?;

            let (counter, args) = extract_task_args(task, &tasks_names[(i + 1)..], self)?;

            tasks.push((name, task, args));

            i += 1 + counter;
        }

        self.detect_circular_dependencies(
            &tasks
                .iter()
                .map(|x| (x.0, x.1))
                .collect::<Vec<(&str, &TaskKind)>>(),
        )?;

        self.run_hook("before_script", None)?;

        let scope = &self.scope;

        for (name, task, args) in tasks {
            self.run_task(name, task, args, Rc::clone(scope))?;
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
        template::render_string,
        variables::ValueKind,
    };

    #[test]
    fn test_process_variables() {
        let mut raw_variables_1 = RawVariables::default();

        raw_variables_1.add("var1", ValueKind::String("value1".to_owned()));
        raw_variables_1.add("var2", ValueKind::String("value2 {{ var1 }}".to_owned()));

        let mut devrcfile = Devrcfile::default();

        devrcfile.process_variables(raw_variables_1).unwrap();

        let mut raw_variables_2 = RawVariables::default();
        raw_variables_2.add("var3", ValueKind::String("value3 {{ var2 }}".to_owned()));

        devrcfile.process_variables(raw_variables_2).unwrap();

        let mut scope = Scope::default();
        scope.process_binding("var1", "value1").unwrap();
        scope.process_binding("var2", "value2 {{ var1 }}").unwrap();
        scope.process_binding("var3", "value3 {{ var2 }}").unwrap();

        assert_eq!(scope, devrcfile.get_scope_copy().unwrap());
    }

    #[test]
    fn test_env_variables() {
        let mut raw_variables_1 = RawVariables::default();

        raw_variables_1.add("var1", ValueKind::String("value1".to_owned()));
        raw_variables_1.add("var2", ValueKind::String("value2 {{ var1 }}".to_owned()));

        let mut devrcfile = Devrcfile::default();

        devrcfile.process_variables(raw_variables_1).unwrap();

        let mut raw_environment = RawEnvironment::default();
        raw_environment.add("env_var1", "value3 {{ var2 }}".to_owned());

        devrcfile.process_env_variables(raw_environment).unwrap();

        let mut scope = Scope::default();

        scope.process_binding("var1", "value1").unwrap();
        scope.process_binding("var2", "value2 {{ var1 }}").unwrap();
        scope.insert_env(
            "env_var1",
            &render_string("env_var", "value3 {{ var2 }}", &scope).unwrap(),
        );

        assert_eq!(scope, devrcfile.get_scope_copy().unwrap());
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
