use std::{fmt, fmt::Display, string::String};

use devrc_plugins::options::PluginOption;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(untagged)]
pub enum InterpreterOption {
    #[default]
    None,
    String(String),
    List(Vec<InterpreterOption>),
    Map(indexmap::IndexMap<String, InterpreterOption>),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginInterpreter {
    pub runtime: String,

    #[serde(default)]
    pub args: Option<Vec<String>>,

    #[serde(default)]
    pub options: indexmap::IndexMap<String, InterpreterOption>,
}

impl Display for PluginInterpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(args) = &self.args.as_ref() {
            write!(f, "{} {}", &self.runtime, &args.join(" "))
        } else {
            write!(f, "{}", &self.runtime)
        }
    }
}

impl From<InterpreterOption> for PluginOption {
    fn from(value: InterpreterOption) -> Self {
        match value {
            InterpreterOption::None => PluginOption::None,
            InterpreterOption::String(value) => PluginOption::String(value),
            InterpreterOption::List(list) => {
                PluginOption::List(list.into_iter().map(|item| item.into()).collect())
            }
            InterpreterOption::Map(map) => PluginOption::Map(
                map.into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect(),
            ),
        }
    }
}
