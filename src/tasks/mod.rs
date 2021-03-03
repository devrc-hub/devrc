use std::{convert::TryFrom, fmt, marker::PhantomData};

use indexmap::IndexMap;

use serde::{Deserialize, Deserializer};

use serde::de::{MapAccess, Visitor};

use crate::{
    config::Config,
    errors::{DevrcError, DevrcResult},
    scope::Scope,
    workshop::Designer,
};

pub mod arguments;
pub mod complex;
pub mod examples;
pub mod exec;
pub mod params;
pub mod params_parser;

pub use crate::tasks::{examples::Examples, exec::ExecKind, params::Params};

use self::{complex::ComplexCommand, params::ParamValue};
use crate::tasks::arguments::TaskArguments;

#[derive(Debug, Deserialize, Clone)]
pub struct FileInclude {
    pub file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteInclude {
    pub remote: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum IncludeKind {
    Empty,
    Simple(String),
    File(FileInclude),
    Remote(RemoteInclude),
    List(Vec<IncludeKind>),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Include {
    include: IncludeKind,
}

// TODO: put `ComplexCommand` into `Box`
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TaskKind {
    Empty,
    Command(String),
    ComplexCommand(ComplexCommand),
    Commands(ExecKind),
    Include(Include),
}

impl Default for TaskKind {
    fn default() -> Self {
        TaskKind::Empty
    }
}

impl TaskKind {
    pub fn get_example(&self) -> Option<String> {
        match self {
            TaskKind::ComplexCommand(ComplexCommand { example, .. }) => example.to_owned(),
            _ => None,
        }
    }

    pub fn get_usage_help(&self, name: &str, designer: &Designer) -> DevrcResult<String> {
        Ok(format!(
            "{}{}{} {}",
            designer.task_name().prefix(),
            name.to_string(),
            designer.task_name().suffix(),
            self.format_parameters_help(&designer)?
        ))
    }

    pub fn format_help(&self) -> DevrcResult<&str> {
        match self {
            TaskKind::Empty => Ok(""),
            TaskKind::Command(_command) => Ok(""),
            TaskKind::ComplexCommand(command) => Ok(command.format_help()),
            TaskKind::Commands(_) => Ok(""),
            TaskKind::Include(_) => Ok(""),
        }
    }

    pub fn format_parameters_help(&self, designer: &Designer) -> DevrcResult<String> {
        match self {
            TaskKind::ComplexCommand(command) => command.format_parameters_help(designer),
            _ => Ok("".to_owned()),
        }
    }

    pub fn is_private(&self) -> bool {
        false
        // if let Some(name) = &self.name {
        //     name.startswith("_")
        // } else {
        //     false
        // }
    }

    pub fn perform(
        &self,
        name: &str,
        parent_scope: &Scope,
        args: &TaskArguments,
        config: &Config,
        designer: &Designer,
    ) -> DevrcResult<()> {
        config.log_level.debug(
            &format!("\n==> Running task: `{:}` ...", &name),
            &designer.banner(),
        );

        match self {
            TaskKind::Empty => return Err(DevrcError::NotImplemented),
            TaskKind::Command(value) => {
                let complex_command = ComplexCommand::from(value);
                complex_command.perform(name, parent_scope, args, &config, &designer)?;
            }
            TaskKind::ComplexCommand(value) => {
                value.perform(name, parent_scope, args, &config, &designer)?;
            }
            TaskKind::Commands(_value) => return Err(DevrcError::NotImplemented),
            TaskKind::Include(_value) => {
                return Err(DevrcError::NotImplemented);
            }
        }

        Ok(())
    }

    // Get list of task dependencies
    pub fn get_dependencies(&self) -> Option<&Vec<String>> {
        if let TaskKind::ComplexCommand(command) = self {
            if !&command.deps.is_empty() {
                return Some(&command.deps);
            }
        };

        None
    }

    pub fn get_parameters(
        &self,
        parts: &[String],
    ) -> DevrcResult<indexmap::IndexMap<String, ParamValue>> {
        match self {
            TaskKind::ComplexCommand(value) => value.get_parameters(&parts),
            _ => Ok(indexmap::IndexMap::new()),
        }
    }

    pub fn has_parameters(&self) -> bool {
        match self {
            TaskKind::ComplexCommand(value) => value.has_parameters(),
            _ => false,
        }
    }
}

//#[derive(Debug, Deserialize, Clone, Default)]
//pub struct Task(TaskKind);

pub type Task = TaskKind;
pub type TaskName = String;

#[derive(Debug, Clone, Default)]
pub struct Tasks {
    pub items: IndexMap<TaskName, Task>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TasksFiles {}

pub fn extract_name_and_params(value: String) -> DevrcResult<(String, Params)> {
    let mut parts = value.splitn(2, ' ');

    let name = parts.next().ok_or(DevrcError::InvalidName)?;

    let mut params: Params = Params::default();

    if let Some(value) = parts.next() {
        params = Params::try_from(value.to_string())?;
    }

    Ok((name.to_string(), params))
}

impl Tasks {
    pub fn add_task(&mut self, name: TaskName, task: Task) -> DevrcResult<()> {
        if let TaskKind::ComplexCommand(mut command) = task {
            let (name, params) = extract_name_and_params(name)?;

            command.setup_name(&name)?;
            command.setup_params(params)?;

            self.items.insert(name, TaskKind::ComplexCommand(command));
        } else {
            self.items.insert(name, task);
        }

        Ok(())
    }

    pub fn find_task(&self, name: &str) -> DevrcResult<&Task> {
        let task = self.items.get(name);

        match task {
            Some(value) => Ok(value),
            None => Err(DevrcError::TaskNotFound),
        }
    }
}

impl<'de> Deserialize<'de> for Tasks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StructVisitor<'de> {
            marker: PhantomData<Tasks>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de> Visitor<'de> for StructVisitor<'de> {
            type Value = Tasks;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct Tasks")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut elements: IndexMap<TaskName, Task> = IndexMap::new();

                while let Some((key, value)) = match access.next_entry::<TaskName, Task>() {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                } {
                    let command = match value {
                        TaskKind::Command(value) => {
                            TaskKind::ComplexCommand(ComplexCommand::from(value))
                        }
                        TaskKind::Commands(commands) => {
                            TaskKind::ComplexCommand(ComplexCommand::from(commands))
                        }
                        _ => value,
                    };
                    elements.insert(key, command);
                }

                Ok(Tasks { items: elements })
            }
        }

        Deserializer::deserialize_map(
            deserializer,
            StructVisitor {
                marker: PhantomData::<Tasks>,
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_task_execute() {}
}
