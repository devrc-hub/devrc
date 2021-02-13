use std::{
    fs::File,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use dotenv;

use crate::{
    errors::{DevrcError, DevrcResult},
    evaluate::Evaluatable,
    scope::Scope,
    template::render_string,
    utils::get_absolute_path,
    variables::ValueKind,
};

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

fn get_default_skip_on_error() -> bool {
    false
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileInclude {
    pub file: PathBuf,

    #[serde(default = "get_default_skip_on_error")]
    pub ignore_errors: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileRemote {
    pub remote: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StringFileInclude(pub String);

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum EnvFile {
    Empty,
    Simple(PathBuf),
    File(FileInclude),
    Remote(FileRemote),
    List(Vec<EnvFile>),
}

impl EnvFile {
    pub fn load(
        &self,
        base_path: Option<&PathBuf>,
    ) -> DevrcResult<indexmap::IndexMap<String, String>> {
        let mut environment: indexmap::IndexMap<String, String> = indexmap::IndexMap::new();

        match self {
            EnvFile::Empty => {}
            EnvFile::Simple(path) => {
                let file = get_absolute_path(&path, base_path)?;
                self.get_from_file(file, &mut environment)?;
            }
            EnvFile::File(FileInclude {
                file,
                ignore_errors,
            }) => match get_absolute_path(&file, base_path) {
                Ok(file) => match self.get_from_file(file, &mut environment) {
                    Err(error) => {
                        if !ignore_errors {
                            return Err(error);
                        }
                    }
                    _ => {}
                },
                Err(error) => {
                    if !ignore_errors {
                        return Err(error);
                    }
                }
            },
            EnvFile::Remote(_) => {
                todo!()
            }
            EnvFile::List(items) => {
                for item in items.into_iter() {
                    for (key, value) in item.load(base_path)? {
                        environment.insert(key, value);
                    }
                }
            }
        }

        Ok(environment)
    }

    pub fn get_from_file(
        &self,
        file: PathBuf,
        environment: &mut indexmap::IndexMap<String, String>,
    ) -> DevrcResult<()> {
        for item in dotenv::from_path_iter(file)? {
            match item {
                Ok((key, value)) => {
                    environment.insert(key, value);
                }
                Err(error) => {
                    return Err(DevrcError::Dotenv(error));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnvFilesWrapper(pub EnvFile);

pub type Environment<T> = indexmap::IndexMap<T, String>;

impl<T> RawEnvironment<T>
where
    T: Evaluatable,
{
    pub fn evaluate(&self, parent_scope: &Scope) -> DevrcResult<Environment<String>> {
        let mut local_scope = parent_scope.clone();
        let mut vars = Environment::default();
        for (key, value) in &self.vars {
            match value.evaluate(&key, &local_scope) {
                Ok(value) => {
                    // local_scope.insert_var(&key, &value);
                    vars.insert(key.clone(), value)
                }
                Err(error) => return Err(error),
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

    use tera::Error as TeraError;

    #[test]
    fn test_string_evaluation() {
        let mut variables: RawEnvironment<String> = RawEnvironment::default();
        variables.add("key1", "key1 value \"{{ parent_scope_var }}\"".to_owned());

        let mut scope = Scope::default();
        scope.insert_var("parent_scope_var", "parent_scope_var_value");

        assert_eq!(variables.evaluate(&scope).unwrap(), {
            let mut control = Environment::default();
            control.insert(
                "key1".to_owned(),
                "key1 value \"parent_scope_var_value\"".to_owned(),
            );
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
                assert_eq!(
                    "Variable `key1` not found in context while rendering \'key2\'",
                    format!("{:}", TeraError::source(&terra_error).unwrap())
                );
            }
            _ => assert!(false),
        }
    }
}
