use crate::{
    errors::{DevrcError, DevrcResult},
    scope::Scope,
};

use std::process::Command;

pub trait CommandExt {
    fn export_scope(&mut self, scope: &Scope) -> DevrcResult<()>;
}

impl CommandExt for Command {
    fn export_scope(&mut self, scope: &Scope) -> DevrcResult<()> {
        if scope.parent.is_some() {
            let parent_scope = (&**(scope.parent.as_ref().unwrap()))
                .try_borrow()
                .map_err(|_| DevrcError::RuntimeError)?;
            for (key, value) in &parent_scope.environment {
                self.env(key, value);
            }
        }

        for (key, value) in &scope.environment {
            self.env(key, value);
        }

        Ok(())
    }
}

pub trait Executor {}
