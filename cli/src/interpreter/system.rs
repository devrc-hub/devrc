use crate::{config::Config, errors::DevrcResult, execute::CommandExt, scope::Scope};
use std::{convert::TryFrom, fmt, fmt::Display, marker::PhantomData};

use std::os::unix::{fs::PermissionsExt, process::ExitStatusExt};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_yaml::{self, Mapping, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command},
};

use crate::errors::DevrcError;

use tempfile::{Builder, NamedTempFile};

pub const DEFAULT_SHELL: &str = "sh";
pub const DEFAULT_SHELL_ARG: &str = "-c";

pub fn get_default_shell() -> String {
    DEFAULT_SHELL.to_string()
}

#[derive(Debug, Clone)]
pub struct SystemShell {
    // pub kind: InterpeterKind,
    // #[serde(alias="shell")]
    pub interpreter: String,
    pub args: Vec<String>,
}

impl TryFrom<String> for SystemShell {
    type Error = DevrcError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::get_from_string(&value).ok_or(DevrcError::InvalidInterpreter)
    }
}

impl SystemShell {
    /// Parse shell string to `Interpreter` struct
    pub fn get_from_string(input: &str) -> Option<SystemShell> {
        let mut parts = input.split(' ');
        let mut args: Vec<String> = Vec::new();

        if let Some(value) = parts.next() {
            #[allow(clippy::while_let_on_iterator)]
            while let Some(arg) = parts.next() {
                args.push(arg.to_owned())
            }

            Some(SystemShell {
                interpreter: value.to_string(),
                args,
            })
        } else {
            None
        }
    }

    pub fn execute(&self, code: &str, scope: &Scope, config: &Config) -> DevrcResult<i32> {
        let mut command = Command::new(&self.interpreter);
        command.export_environment(&scope.environment)?;

        if let Some(value) = &config.current_dir {
            command.current_dir(value);
        }

        for arg in &self.args {
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
                        return Err(DevrcError::Code { code });
                    }
                } else {
                    println!("Process terminated by signal");
                    return Err(DevrcError::Signal);
                }
            }
            Err(io_error) => {
                return Err(DevrcError::IoError(io_error));
            }
        }
        Ok(0)
    }

    pub fn execute_script(&self, code: &str, scope: &Scope, config: &Config) -> DevrcResult<i32> {
        let (script_path, _tmp) = create_script_file(code)?;
        set_execute_permission(&script_path)?;

        let mut command = Command::new(&script_path);

        command.export_environment(&scope.environment)?;

        if let Some(value) = &config.current_dir {
            command.current_dir(value);
        }

        for arg in &self.args {
            command.arg(arg);
        }

        command.arg(code);

        // Handle signals
        match command.status() {
            Ok(exit_status) => {
                if let Some(code) = exit_status.code() {
                    if code != 0 {
                        // Raise runtime error
                        return Err(DevrcError::Code { code });
                    }
                } else {
                    println!("Process terminated by signal");
                    return Err(DevrcError::Signal);
                }
            }
            Err(io_error) => {
                return Err(DevrcError::IoError(io_error));
            }
        }
        Ok(0)
    }
}

impl Default for SystemShell {
    fn default() -> Self {
        SystemShell {
            interpreter: DEFAULT_SHELL.to_owned(),
            args: vec![DEFAULT_SHELL_ARG.to_owned()],
        }
    }
}

impl Display for SystemShell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", &self.interpreter, &self.args.join(" "))
    }
}

impl<'de> Deserialize<'de> for SystemShell {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StructVisitor<T>(PhantomData<T>);

        #[derive(Deserialize, Debug)]
        pub struct TempInterpreter {
            #[serde(alias = "shell")]
            pub interpreter: String,
            pub args: Vec<String>,
        }

        impl<'de, T> Visitor<'de> for StructVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = SystemShell;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Formatter::write_str(formatter, "struct Interpreter")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match SystemShell::get_from_string(value) {
                    Some(value) => Ok(value),
                    None => Err(de::Error::custom("invalid interpreter")),
                }
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                // TODO: we can simplify this code into few lines
                let mut elements = Mapping::new();

                while let Some((key, value)) = match access.next_entry::<Value, Value>() {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                } {
                    elements.insert(key, value);
                }

                let interpreter: TempInterpreter = serde_yaml::from_value(elements.into())
                    .map_err(|_| de::Error::custom("invalid interpreter"))?;

                Ok(SystemShell {
                    interpreter: interpreter.interpreter,
                    args: interpreter.args,
                })
            }
        }

        let visitor = StructVisitor(PhantomData::<SystemShell>);
        deserializer.deserialize_any(visitor)
    }
}

fn create_script_file(script: &str) -> DevrcResult<(PathBuf, NamedTempFile)> {
    let tmp = Builder::new().prefix("devrc").tempfile()?;

    let path = tmp.path().to_path_buf();

    let mut f = fs::File::create(&path).unwrap();

    f.write_all(script.as_bytes())?;

    Ok((path, tmp))
}

fn set_execute_permission(path: &Path) -> DevrcResult<()> {
    let mut permissions = fs::metadata(path)?.permissions();

    permissions.set_mode(permissions.mode() | 0o100);

    // set the new permissions
    fs::set_permissions(path, permissions).map_err(DevrcError::IoError)
}

#[allow(dead_code)]
fn signal_from_exit_status(exit_status: process::ExitStatus) -> Option<i32> {
    exit_status.signal()
}
