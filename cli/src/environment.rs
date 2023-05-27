use serde::Deserialize;

use crate::{errors::DevrcResult, evaluate::Evaluatable, scope::Scope};

pub type Environment<T> = indexmap::IndexMap<T, String>;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
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

impl<T> RawEnvironment<T>
where
    T: Evaluatable,
{
    pub fn evaluate(&self, parent_scope: &Scope) -> DevrcResult<Environment<String>> {
        let local_scope = parent_scope.clone();
        let mut vars = Environment::default();
        for (key, value) in &self.vars {
            match value.evaluate(key, &local_scope) {
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::errors::DevrcError;
//     use std::error::Error as StdError;
//     use tera::Error as TeraError;
// }
