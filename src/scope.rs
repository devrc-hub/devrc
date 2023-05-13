use std::{cell::RefCell, convert::TryFrom};

use tera::Context;

use crate::{
    environment::RawEnvironment,
    errors::{DevrcError, DevrcResult},
    evaluate::Evaluatable,
    variables::{self, RawVariables, VariableKey, VariableValue, Variables},
};
use std::rc::Rc;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Scope {
    pub variables: Variables,
    pub environment: indexmap::IndexMap<String, String>,
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub root: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    /// Add variable to scope
    pub fn insert_var(&mut self, key: VariableKey, value: VariableValue) -> Option<VariableValue> {
        self.variables.insert(key, value)
    }

    pub fn process_binding(&mut self, key: &str, value: &str) -> DevrcResult<()> {
        self.variables.insert(
            VariableKey::try_from(key.to_string())?,
            VariableValue::new(key, value).with_render_value(self)?,
        );
        Ok(())
    }

    /// Add environment variable to scope
    pub fn insert_env(&mut self, key: &str, value: &str) -> Option<String> {
        self.environment.insert(key.to_owned(), value.to_owned())
    }

    pub fn get_var(&self, key: &VariableKey) -> Option<&VariableValue> {
        self.variables.get(key)
    }

    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }

    pub fn process_raw_vars(&mut self, variables: &RawVariables) -> DevrcResult<()> {
        for (original_key, original_value) in &variables.vars {
            match original_value {
                variables::ValueKind::None => return Err(DevrcError::EmptyVariable),
                variables::ValueKind::String(inner) => {
                    let key = VariableKey::try_from(original_key.clone())?;

                    let value: VariableValue = if key.raw {
                        VariableValue::new(original_key, inner).as_raw()?
                    } else {
                        VariableValue::new(original_key, inner).with_render_value(self)?
                    };

                    self.variables.insert(key.clone(), value.clone());

                    if key.set_global && self.parent.is_some() {
                        let mut parent_scope = (&**(self.parent.as_ref().unwrap()))
                            .try_borrow_mut()
                            .map_err(|_| DevrcError::RuntimeError)?;
                        parent_scope.insert_var(key, value);
                    }
                }
                _ => return Err(DevrcError::VariableTypeNotImplemented),
            }
        }
        Ok(())
    }

    pub fn process_raw_env_vars(&mut self, variables: &RawEnvironment<String>) -> DevrcResult<()> {
        for (key, value) in &variables.vars {
            let result = value.evaluate(key, self);
            match result {
                Ok(rendered_value) => {
                    self.environment
                        .insert(key.to_owned(), rendered_value.to_owned());
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

}

impl Clone for Scope {
    fn clone(&self) -> Self {
        let mut scope = Scope {
            parent: self.parent.clone(),
            ..Default::default()
        };

        for (name, value) in self.variables.iter() {
            scope.insert_var((*name).clone(), (*value).clone());
        }

        for (name, value) in &self.environment {
            scope.insert_env(name, value);
        }

        scope
    }
}

impl TryFrom<&Scope> for Context {
    type Error = DevrcError;

    fn try_from(source: &Scope) -> Result<Self, Self::Error> {
        let mut context: Context = Self::new();

        let mut ancestors = Vec::new();

        let mut parent_link = source.parent.clone();

        loop {
            if parent_link.is_none() {
                break

            }
            ancestors.push(Rc::clone(&parent_link.clone().unwrap()));

            parent_link = {
                let parent_scope = (&**(parent_link.as_ref().unwrap())).try_borrow().map_err(|_| DevrcError::RuntimeError)?;

                if parent_scope.parent.is_none() {
                    break
                }
                parent_scope.parent.clone()
            }
        }

        ancestors.reverse();

        for ancestor in ancestors {

            let scope = ancestor.try_borrow()
                .map_err(|_| DevrcError::RuntimeError)?;

            for (key, value) in &scope.variables {
                 context.insert(key.get_name(), &value.get_rendered_value());
            }
        }



        for (key, value) in &source.variables {
            context.insert(key.get_name(), &value.get_rendered_value());
        }
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope() {
        let mut scope = Scope::default();

        scope.insert_var(
            VariableKey::try_from("key1".to_string()).unwrap(),
            VariableValue::new("key1", "value1"),
        );

        scope.insert_env("env_var_1", "env_var_2_val");

        assert_eq!(
            scope.get_var(&VariableKey::try_from("key".to_string()).unwrap()),
            None
        );

        assert_eq!(
            scope.get_var(&VariableKey::try_from("key1".to_string()).unwrap()),
            Some(&VariableValue::new("key1", "value1"))
        );

        assert_eq!(
            scope.get_env_var("env_var_1"),
            Some(&"env_var_2_val".to_owned())
        );
    }
}
