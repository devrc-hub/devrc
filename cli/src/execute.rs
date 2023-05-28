use crate::errors::DevrcResult;

use std::process::Command;

pub trait CommandExt {
    fn export_environment(
        &mut self,
        environment: &indexmap::IndexMap<String, String>,
    ) -> DevrcResult<()>;
}

impl CommandExt for Command {
    fn export_environment(
        &mut self,
        environment: &indexmap::IndexMap<String, String>,
    ) -> DevrcResult<()> {
        for (key, value) in environment.into_iter() {
            self.env(key, value);
        }

        Ok(())
    }
}

pub trait Executor {}
