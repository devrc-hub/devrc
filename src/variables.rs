use std::convert::TryFrom;

use crate::{
    errors::{
        DevrcError::{self},
        DevrcResult,
    },
    scope::Scope,
    template::render_string,
    variables_parser::parse_key,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Http {
    fetch: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct File {
    file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Computable {
    #[allow(dead_code)]
    exec: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ValueKind {
    None,
    String(String),
    Http(Http),
    File(File),
    Computable(Computable),
}

impl Default for ValueKind {
    fn default() -> Self {
        ValueKind::None
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RawVariables {
    #[serde(flatten)]
    pub vars: indexmap::IndexMap<String, ValueKind>,
}

pub type Variables = indexmap::IndexMap<VariableKey, VariableValue>;
pub trait VariablesTrait {}

impl VariablesTrait for Variables {}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct VariableKey {
    pub original: String,
    pub name: String,
    pub set_global: bool,
}

// impl From<String> for VariableKey {
//     fn from(source: String) -> Self {
//         VariableKey(source)
//     }
// }

impl TryFrom<String> for VariableKey {
    type Error = DevrcError;

    fn try_from(source: String) -> Result<Self, Self::Error> {
        parse_key(&source)
    }
}

impl VariableKey {
    pub fn get_name(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct VariableValue {
    pub name: String,
    pub raw: String,
    pub rendered: Option<String>,
}

impl VariableValue {
    pub fn new(name: &str, raw: &str) -> Self {
        Self {
            name: name.to_owned(),
            raw: raw.to_owned(),
            rendered: None,
        }
    }

    pub fn get_rendered_value(&self) -> String {
        self.rendered.clone().unwrap_or_default()
    }

    pub fn render_value(&mut self, name: &str, scope: &Scope) -> DevrcResult<()> {
        self.rendered = Some(render_string(name, &self.raw, scope)?);
        Ok(())
    }

    pub fn with_render_value(mut self, scope: &Scope) -> DevrcResult<Self> {
        self.rendered = Some(render_string(&self.name, &self.raw, scope)?);
        Ok(self)
    }
}

impl RawVariables {
    pub fn add(&mut self, name: &str, value: ValueKind) {
        self.vars.insert(name.to_owned(), value);
    }
}

impl Default for RawVariables {
    fn default() -> Self {
        let vars = indexmap::IndexMap::new();
        Self { vars }
    }
}

impl From<Vec<(String, String)>> for RawVariables {
    fn from(items: Vec<(String, String)>) -> Self {
        Self {
            vars: items
                .iter()
                .map(move |x| (x.0.clone(), ValueKind::String(x.1.clone())))
                .collect::<indexmap::IndexMap<String, ValueKind>>(),
        }
    }
}

impl ValueKind {
    pub fn evaluate(&self, name: &str, scope: &Scope) -> DevrcResult<String> {
        match self {
            Self::String(template) => render_string(name, template, scope),
            Self::None => Err(DevrcError::EmptyVariable),
            Self::Http(_) => Ok("TODO: replace me".to_owned()),
            Self::File(_) => Ok("TODO: replace me".to_owned()),
            Self::Computable(_) => Ok("TODO: replace me".to_owned()),
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     use crate::errors::DevrcError;
//     use std::error::Error as StdError;

//     use tera::Error as TeraError;

//     #[test]
//     fn test_file_variable() {}

//     #[test]
//     fn test_http_variable() {}

//     #[test]
//     fn test_computable_variable() {}
// }
