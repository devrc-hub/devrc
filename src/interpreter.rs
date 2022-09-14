use std::{convert::TryFrom, fmt, fmt::Display, marker::PhantomData};

use crate::{config::Config, errors::DevrcResult, execute::CommandExt, scope::Scope};

#[cfg(feature = "deno")]
use crate::denoland::execute_deno_code;

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

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum DenoPermissionParamValue {
    All,
    String(String),
    List(Vec<String>),
}

impl Default for DenoPermissionParamValue {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Deserialize, Clone)]
// #[serde(untagged)]
pub enum DenoPermission {
    #[serde(rename = "disable-all")]
    DisableAll,
    #[serde(rename = "allow-all")]
    AllowAll,
    #[serde(rename = "allow-env")]
    AllowEnv(Vec<String>),
    #[serde(rename = "allow-hrtime")]
    AllowHrtime,
    #[serde(rename = "allow-ffi")]
    AllowFFI(Vec<String>),
    #[serde(rename = "allow-run")]
    AllowRun(Vec<String>),

    #[serde(rename = "allow-write-all")]
    AllowWriteAll,
    #[serde(rename = "allow-read-all")]
    AllowReadAll,
    #[serde(rename = "allow-net-all")]
    AllowNetAll,

    #[serde(rename = "allow-run-all")]
    AllowRunAll,
    #[serde(rename = "allow-ffi-all")]
    AllowFFIAll,

    #[serde(rename = "allow-net")]
    AllowNet(Vec<String>),
    #[serde(rename = "allow-read")]
    AllowRead(Vec<String>),
    #[serde(rename = "allow-write")]
    AllowWrite(Vec<String>),
}

#[derive(Debug, Deserialize, Clone)]
pub enum RuntimeName {
    #[serde(rename = "deno-runtime")]
    Deno,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DenoRuntime {
    pub permissions: Option<Vec<DenoPermission>>,
    pub runtime: RuntimeName,
    pub args: Option<Vec<String>>,
}

#[cfg(not(feature = "deno"))]
pub fn execute_deno_code(
    _code: &str,
    _permissions: &Option<Vec<DenoPermission>>,
) -> DevrcResult<i32> {
    Err(DevrcError::DenoFeatureRequired)
}

impl DenoRuntime {
    pub fn execute(&self, code: &str, _scope: &Scope, _config: &Config) -> DevrcResult<i32> {
        match execute_deno_code(code, &self.permissions) {
            Ok(exit_status) => {
                if exit_status != 0 {
                    // Raise runtime error
                    Err(DevrcError::Code { code: exit_status })
                } else {
                    Ok(0)
                }
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum InterpreterKind {
    DenoRuntime(DenoRuntime),
    Internal(SystemShell),
}

impl InterpreterKind {
    pub fn execute(&self, code: &str, scope: &Scope, config: &Config) -> DevrcResult<i32> {
        match self {
            InterpreterKind::DenoRuntime(deno_interpreter) => {
                deno_interpreter.execute(code, scope, config)
            }
            InterpreterKind::Internal(internal_shell) => {
                internal_shell.execute(code, scope, config)
            }
        }
    }
}

impl Default for InterpreterKind {
    fn default() -> Self {
        InterpreterKind::Internal(SystemShell::default())
    }
}

impl Display for InterpreterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterKind::DenoRuntime(interpreter) => {
                write!(f, "{:?}", interpreter)
            }
            InterpreterKind::Internal(interpreter) => {
                write!(f, "{}", interpreter)
            }
        }
    }
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
        command.export_scope(scope)?;

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

        command.export_scope(scope)?;

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

pub trait ShebangDetector {
    fn get_interpreter_from_shebang(&self) -> Option<SystemShell>;
}

impl ShebangDetector for String {
    fn get_interpreter_from_shebang(&self) -> Option<SystemShell> {
        let first_line = self.lines().next().unwrap_or("");

        if !first_line.starts_with("#!") {
            return None;
        }

        let mut parts = first_line[2..].splitn(2, |c| c == ' ' || c == '\t');

        if let Some(value) = parts.next() {
            let mut args = Vec::new();

            if let Some(value) = parts.next().map(|arg| arg.to_owned()) {
                args.push(value)
            };

            Some(SystemShell {
                interpreter: value.to_owned(),
                args,
            })
        } else {
            None
        }
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
    let mut permissions = fs::metadata(&path)?.permissions();

    permissions.set_mode(permissions.mode() | 0o100);

    // set the new permissions
    fs::set_permissions(&path, permissions).map_err(DevrcError::IoError)
}

#[allow(dead_code)]
fn signal_from_exit_status(exit_status: process::ExitStatus) -> Option<i32> {
    exit_status.signal()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {}
}
