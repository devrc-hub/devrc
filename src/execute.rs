use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    thread,
    time::Duration,
};

use crate::{
    config::Config,
    errors::{DevrcError, DevrcResult},
    interpreter::{Interpreter, ShebangDetector},
    scope::Scope,
};
use run_script::{IoOptions, ScriptOptions};
use std::os::unix::{fs::PermissionsExt, process::ExitStatusExt};
use tempfile::Builder;

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
