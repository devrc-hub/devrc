use crate::{
    errors::{DevrcResult},
    scope::Scope,
};

use std::{process::Command};


pub trait CommandExt {
    fn export_scope(&mut self, scope: &Scope) -> DevrcResult<()>;
}

impl CommandExt for Command {
    fn export_scope(&mut self, scope: &Scope) -> DevrcResult<()> {
        for (key, value) in &scope.environment {
            self.env(key, value);
        }

        Ok(())
    }
}
