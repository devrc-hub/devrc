use serde::Deserialize;

use crate::{errors::DevrcResult, evaluate::Evaluatable, scope::Scope, template::render_string, variables::ValueKind};

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct RawEnvironment<T> {
    #[serde(flatten)]
    pub vars: indexmap::IndexMap<String, T>,
}

impl<T> Default for RawEnvironment<T> {
    fn default() -> Self {
        let vars = indexmap::IndexMap::new();
        Self { vars }
    }
}


#[derive(Debug, Deserialize, Clone)]
pub struct FileInclude {
    pub file: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileRemote {
    pub remote: String
}


#[derive(Debug, Deserialize, Clone)]
pub struct StringFileInclude(pub String);


#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum EnvFiles {
    Empty,
    Simple(StringFileInclude),
    File(FileInclude),
    Remote(FileRemote),
    List(Vec<EnvFiles>)
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnvFilesWrapper(pub EnvFiles);


pub type Environment<T> = indexmap::IndexMap<T, String>;


impl<T> RawEnvironment<T>
where T: Evaluatable{
    pub fn evaluate(&self, parent_scope: &Scope) -> DevrcResult<Environment<String>>
    {
        let mut local_scope = parent_scope.clone();
        let mut vars = Environment::default();
        for (key, value) in &self.vars {
            match value.evaluate(&key, &local_scope) {
                Ok(value) => {
                    // local_scope.insert_var(&key, &value);
                    vars.insert(key.clone(), value)
                },
                Err(error) => return Err(error)
            };

        }
        Ok(vars)
    }

    pub fn add(&mut self, name: &str, value: T) {
        self.vars.insert(name.to_owned(), value);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::errors::DevrcError;
    use std::error::Error as StdError;

    use super::*;

    use tera::{Error as TeraError};

    #[test]
    fn test_string_evaluation() {
        let mut variables: RawEnvironment<String> = RawEnvironment::default();
        variables.add("key1", "key1 value \"{{ parent_scope_var }}\"".to_owned());

        let mut scope = Scope::default();
        scope.insert_var("parent_scope_var", "parent_scope_var_value");

        assert_eq!(variables.evaluate(&scope).unwrap(), {
            let mut control = Environment::default();
            control.insert("key1".to_owned(), "key1 value \"parent_scope_var_value\"".to_owned());
            control
        });
    }


    #[test]
    fn test_string_evaluation_error() {
        let mut variables: RawEnvironment<String> = RawEnvironment::default();

        variables.add("key1", "key1 value".to_owned());
        variables.add("key2", "key2 value \"{{ key1 }}\"".to_owned());

        let scope = Scope::default();

        match variables.evaluate(&scope) {
            Err(DevrcError::RenderError(terra_error)) => {
                assert_eq!("Variable `key1` not found in context while rendering \'key2\'",
                           format!("{:}", TeraError::source(&terra_error).unwrap()));
            },
            _ => assert!(false)
        }
    }
}
