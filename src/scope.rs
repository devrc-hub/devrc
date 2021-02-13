use std::collections::HashMap;

use tera::Context;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Scope {
    pub variables: HashMap<String, String>,
    pub environment: HashMap<String, String>,
}

impl Scope {
    /// Add variable to scope
    pub fn insert_var(&mut self, key: &str, value: &str) -> Option<String> {
        self.variables.insert(key.to_owned(), value.to_owned())
    }

    /// Add environment variable to scope
    pub fn insert_env(&mut self, key: &str, value: &str) -> Option<String> {
        self.environment.insert(key.to_owned(), value.to_owned())
    }

    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }

    pub fn clone(&self) -> Self {
        // TODO: rewrite and remove copy
        let mut scope = Self::default();

        for (name, value) in &self.variables {
            scope.insert_var(name, value);
        }

        for (name, value) in &self.environment {
            scope.insert_env(name, value);
        }

        scope
    }
}

impl From<&Scope> for Context {
    fn from(source: &Scope) -> Self {
        let mut context: Context = Self::new();

        for (key, value) in &source.variables {
            context.insert(key, value);
        }
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope() {
        let mut scope = Scope::default();

        scope.insert_var("key1", "value1");

        scope.insert_env("env_var_1", "env_var_2_val");

        assert_eq!(scope.get_var("key"), None);

        assert_eq!(scope.get_var("key1"), Some(&"value1".to_owned()));

        assert_eq!(
            scope.get_env_var("env_var_1"),
            Some(&"env_var_2_val".to_owned())
        );
    }
}
