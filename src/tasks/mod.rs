use std::{fmt, marker::PhantomData};

use indexmap::IndexMap;
use serde;

use serde::{Deserialize, Deserializer};

use serde::de::{MapAccess, SeqAccess, Visitor};

use crate::{
    config::Config,
    errors::{DevrcError, DevrcResult},
    scope::Scope,
};

pub mod complex;
pub mod examples;
pub mod exec;
pub mod params;

pub use crate::tasks::{examples::Examples, exec::ExecKind, params::Params};

use self::complex::ComplexCommand;

#[derive(Debug, Clone, Default)]
pub struct Commands {
    pub items: Vec<TaskKind>,
}

impl Commands {
    pub fn push(&mut self, value: TaskKind) -> DevrcResult<()> {
        self.items.push(value);
        Ok(())
    }
}

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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TaskKind {
    Empty,
    Command(String),
    ComplexCommand(ComplexCommand),
    Commands(Commands),
    Include(Include),
}

impl Default for TaskKind {
    fn default() -> Self {
        TaskKind::Empty
    }
}

impl TaskKind {
    pub fn format_help(&self) -> DevrcResult<&str> {
        match self {
            TaskKind::Empty => Ok("doc string"),
            TaskKind::Command(_command) => {
                return Err(DevrcError::NotImplemented);
            }
            TaskKind::ComplexCommand(command) => Ok(command.format_help()),
            TaskKind::Commands(_) => Ok("doc string"),
            TaskKind::Include(_) => Ok("doc string"),
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
        params: &[String],
        config: &Config,
    ) -> DevrcResult<()> {
        println!("\n==> Running task: `{:}` ...", &name);
        match self {
            TaskKind::Empty => return Err(DevrcError::NotImplemented),
            TaskKind::Command(value) => {
                let complex_command = ComplexCommand::from(value);
                complex_command.perform(name, parent_scope, params, &config)?;
            }
            TaskKind::ComplexCommand(value) => {
                value.perform(name, parent_scope, params, &config)?;
            }
            TaskKind::Commands(_value) => return Err(DevrcError::NotImplemented),
            TaskKind::Include(value) => {
                dbg!(value);
                return Err(DevrcError::NotImplemented);
            }
        }

        Ok(())
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

impl Tasks {
    pub fn add_task(&mut self, name: TaskName, task: Task) {
        self.items.insert(name, task);
    }

    pub fn find_task(&self, name: &str) -> DevrcResult<Task> {
        let task = self.items.get(name);

        match task {
            Some(value) => Ok(value.clone()),
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

                while let Some((key, value)) = match access.next_entry() {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                } {
                    // TODO: process key with params
                    elements.insert(key, value);
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

// macro_rules! task_value_result_map {

//     ($var:ident, $from:ty => $to:expr, $value:ident, $return:expr) => {

//         if let Ok($value) = Result::map(
//             <$from as Deserialize>::deserialize(
//                 ContentRefDeserializer::<D::Error>::new(&$var)
//             ),
//             $to
//         ){
//             let value = match $value {
//                 TaskKind::Command(inner) => {
//                     TaskKind::ComplexCommand(ComplexCommand::from(inner))
//                 },
//                 value => {
//                     value
//                 }
//             };

//             return Ok(value);
//         }
//     };
//     ($var:ident, $from:ty => $to:expr) => {
//         task_value_result_map!{$var, $from => $to, value, value}
//     }
// }

// impl<'de> Deserialize<'de> for TaskKind {

//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de> {

//         let content = match <Content as Deserialize>::deserialize(deserializer){
//             Ok(val) => val,
//             Err(error) => {
//                 return Err(error);
//             }

//         };

//         if let Ok(value) = match Deserializer::deserialize_any(
//             ContentRefDeserializer::<D::Error>::new(&content),
//             UntaggedUnitVisitor::new("TaskValue", "Empty")){
//             Ok(value) => Ok(TaskKind::Empty),
//             Err(error) => Err(error)
//         }{
//             return Ok(value);
//         }

//         task_value_result_map!{content, String => TaskKind::Command, value, value};
//         task_value_result_map!{content, Include => TaskKind::Include, value, value};
//         task_value_result_map!{content, ComplexCommand => TaskKind::ComplexCommand, value, value};
//         task_value_result_map!{content, Commands => TaskKind::Commands, value, value};
//         // task_value_result_map!{content, Vec<TaskKind> => TaskKind::Commands, value, value};

//         Ok(TaskKind::Empty)
//     }
// }

impl<'de> Deserialize<'de> for Commands {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StructVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for StructVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Commands;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct Tasks")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut commands = Commands::default();

                while let Some(value) = match seq.next_element() {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                } {
                    let _ = commands.push(value);
                }

                Ok(commands)
            }
        }

        let visitor = StructVisitor(PhantomData::<Commands>);

        deserializer.deserialize_seq(visitor)

        // Ok(Commands::default())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_task_execute() {}
}
