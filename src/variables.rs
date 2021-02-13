use crate::{
    errors::{
        DevrcError::{self},
        DevrcResult,
    },
    scope::Scope,
    template::render_string,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Http {
    fetch: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct File {
    file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Computable {
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

pub type Variables<T> = indexmap::IndexMap<T, String>;

// #[derive(Debug, Clone, Default, PartialEq)]
// pub struct EvaludatedVariables{

//     pub vars: indexmap::IndexMap<String, String>
// }

impl Default for RawVariables {
    fn default() -> Self {
        let vars = indexmap::IndexMap::new();
        Self { vars }
    }
}

impl RawVariables {
    pub fn evaluate(&self, parent_scope: &Scope) -> DevrcResult<Variables<String>> {
        let mut local_scope = parent_scope.clone();
        let mut vars = Variables::default();
        for (key, value) in &self.vars {
            match value.evaluate(&key, &local_scope) {
                Ok(value) => {
                    local_scope.insert_var(&key, &value);
                    vars.insert(key.clone(), value)
                }
                Err(error) => return Err(error),
            };
        }
        Ok(vars)
    }

    pub fn add(&mut self, name: &str, value: ValueKind) {
        self.vars.insert(name.to_owned(), value);
    }
}

impl ValueKind {
    pub fn evaluate(&self, name: &str, scope: &Scope) -> DevrcResult<String> {
        match self {
            Self::String(template) => render_string(name, &template, scope),
            Self::None => Err(DevrcError::EmptyVariable),
            Self::Http(_) => Ok("TODO: replace me".to_owned()),
            Self::File(_) => Ok("TODO: replace me".to_owned()),
            Self::Computable(_) => Ok("TODO: replace me".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::errors::DevrcError;
    use std::error::Error as StdError;

    use tera::Error as TeraError;

    #[test]
    fn test_string_evaluation() {
        let mut variables = RawVariables::default();

        variables.add(
            "key1",
            ValueKind::String("key1 value \"{{ parent_scope_var }}\"".to_owned()),
        );
        variables.add(
            "key2",
            ValueKind::String("key2 value \"{{ key1 }}\"".to_owned()),
        );

        // variables.vars.insert("key1".to_owned(), ValueKind::String("key1 value \"{{ parent_scope_var }}\"".to_owned()));
        // variables.vars.insert("key2".to_owned(), ValueKind::String("key2 value \"{{ key1 }}\"".to_owned()));

        let mut scope = Scope::default();

        scope.insert_var("parent_scope_var", "parent_scope_var_value");

        assert_eq!(variables.evaluate(&scope).unwrap(), {
            let mut control = Variables::default();
            control.insert(
                "key1".to_owned(),
                "key1 value \"parent_scope_var_value\"".to_owned(),
            );
            control.insert(
                "key2".to_owned(),
                "key2 value \"key1 value \"parent_scope_var_value\"\"".to_owned(),
            );
            control
        });
    }

    #[test]
    fn test_string_evaluation_error() {
        let mut variables = RawVariables::default();

        variables.add(
            "key1",
            ValueKind::String("key1 value \"{{ parent_scope_var }}\"".to_owned()),
        );
        variables.add(
            "key2",
            ValueKind::String("key2 value \"{{ key1 }}\"".to_owned()),
        );

        let scope = Scope::default();

        match variables.evaluate(&scope) {
            Err(DevrcError::RenderError(terra_error)) => {
                assert_eq!(
                    "Variable `parent_scope_var` not found in context while rendering \'key1\'",
                    format!("{:}", TeraError::source(&terra_error).unwrap())
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_file_variable() {}

    #[test]
    fn test_http_variable() {}

    #[test]
    fn test_computable_variable() {}
}
