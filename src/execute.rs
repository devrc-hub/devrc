use std::{fs, io::{self, Write}, path::{Path, PathBuf}, process::{self, Command, Stdio}, thread, time::Duration};

use crate::{errors::{DevrcError, DevrcResult}, config::Config, scope::Scope, interpreter::{Interpreter, ShebangDetector}};
use run_script::{ScriptOptions, IoOptions};
use tempfile::Builder;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;


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
