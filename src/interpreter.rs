use core::time;
use std::{fmt::Display, marker::PhantomData};
use std::fmt;

use crate::{config::Config, errors::DevrcResult, execute::CommandExt, scope::Scope};
use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor};
use std::{fs, io::{self, Write}, path::{Path, PathBuf}, process::{self, Command, Stdio}, thread, time::Duration};

use crate::{errors::{DevrcError}};
use run_script::{ScriptOptions, IoOptions};
use tempfile::Builder;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use tempfile::NamedTempFile;

pub const DEFAULT_SHELL: &str = "sh";
pub const DEFAULT_SHELL_ARG: &str = "-c";


pub fn get_default_shell() -> String {
    DEFAULT_SHELL.to_string()
}

#[derive(Debug, Clone)]
pub struct Interpreter{

    pub interpreter: String,
    pub args: Vec<String>
}

impl Interpreter {

    /// Parse shell string to `Interpreter` struct
    pub fn get_from_string(input: &str) -> Option<Interpreter>{
        let mut parts = input.split(' ');
        let mut args:Vec<String> = Vec::new();

        if let Some(value) = parts.next() {

            while let Some(arg) = parts.next() {
                args.push(arg.to_owned())
            }

            Some(Interpreter {
                interpreter: value.to_string(),
                args
            })
        } else {
            None
        }
    }

    pub fn execute(&self, code: &str,  scope: &Scope, config: &Config) -> DevrcResult<()>{
        let mut command = Command::new(&self.interpreter);
        command.export_scope(&scope)?;

        if let Some(value) = &config.current_dir {
            command.current_dir(value);
        }

        for arg in &self.args{
            command.arg(arg);
        }

        command.arg(code);

        // command.stdin(Stdio::null());
        // command.stdout(Stdio::null());
        // command.stderr(Strio::null());

        // Handle signals
        match command.status() {
            Ok(exit_status) => {
                if let Some(code) = exit_status.code() {
                    if code != 0 {
                        // Raise runtime error
                         return Err(DevrcError::Code{
                            code: code
                        });
                    }
                } else {
                    println!("Process terminated by signal");
                    return Err(DevrcError::Signal);

                }
            },
            Err(io_error) => {
                return Err(DevrcError::IoError(io_error));
            }
        }
        Ok(())
    }

    pub fn execute_script(&self, code: &str, scope: &Scope, config: &Config) -> DevrcResult<()>{
        let (script_path, _tmp) = create_script_file(code)?;
        set_execute_permission(&script_path)?;

        let mut command = Command::new(&script_path);

        command.export_scope(&scope)?;

        if let Some(value) = &config.current_dir {
            command.current_dir(value);
        }

        for arg in &self.args{
            command.arg(arg);
        }

        command.arg(code);

        // Handle signals
        match command.status() {
            Ok(exit_status) => {
                if let Some(code) = exit_status.code() {
                    if code != 0 {
                        // Raise runtime error
                        return Err(DevrcError::Code{
                            code: code
                        });
                    }
                } else {
                    println!("Process terminated by signal");
                    return Err(DevrcError::Signal);

                }
            },
            Err(io_error) => {
                return Err(DevrcError::IoError(io_error));
            }
        }
        Ok(())
    }
}


impl Default for Interpreter {
    fn default() -> Self {
        Interpreter {
            interpreter: DEFAULT_SHELL.to_owned(),
            args: vec![DEFAULT_SHELL_ARG.to_owned()]
        }
    }
}

impl Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", &self.interpreter, &self.args.join(" "))
    }
}


pub trait ShebangDetector {
    fn get_interpreter_from_shebang(&self) -> Option<Interpreter>;
}


impl ShebangDetector for String {

    fn get_interpreter_from_shebang(&self) -> Option<Interpreter>{

        let first_line = self.lines().next().unwrap_or("");

        if !first_line.starts_with("#!") {
            return None;
        }

        let mut parts = first_line[2..].splitn(2, |c| {c == ' ' || c == '\t'});

        if let Some(value) = parts.next(){
            let mut args = Vec::new();

            if let Some(value) = parts.next().map(|arg| {arg.to_owned()}) {
                args.push(value)
            };

            Some(Interpreter{
                interpreter: value.to_owned(),
                args
            })
        } else {
            None
        }
    }
}


impl<'de> Deserialize<'de> for Interpreter {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {

        struct StructVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for StructVisitor<T>
        where T: Deserialize<'de>
        {
            type Value = Interpreter;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter,
            ) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct Interpreter")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Interpreter::get_from_string(value) {
                    Some(value) => {Ok(value)},
                    None => {Err(de::Error::custom("invalid interpreter"))}
                }
                // Ok(Interpreter::get_from_string(value))
            }
        }

        let visitor = StructVisitor(PhantomData::<Interpreter>);
        deserializer.deserialize_str(visitor)
    }
}



fn create_script_file(script: &str) -> DevrcResult<(PathBuf, NamedTempFile)>{

    let tmp = Builder::new()
        .prefix("devrc")
        .tempfile()?;

    let path = tmp.path().to_path_buf();

    let mut f = fs::File::create(&path).unwrap();

    f.write_all(script.as_bytes())?;

    // dbg!(&path);
    Ok((path, tmp))
}


fn set_execute_permission(path: &Path) -> DevrcResult<()>{
    let mut permissions = fs::metadata(&path)?.permissions();

    permissions.set_mode(permissions.mode() | 0o100);

    // set the new permissions
    fs::set_permissions(&path, permissions).map_err(|error|{
        DevrcError::IoError(error)
    })
}


fn signal_from_exit_status(exit_status: process::ExitStatus) -> Option<i32> {
    exit_status.signal()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {

    }
}