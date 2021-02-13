use std::{fs, path::PathBuf};

use crate::{
    config::RawConfig,
    devrcfile::Devrcfile,
    errors::{DevrcError, DevrcResult},
    include::IncludeFilesWrapper,
    scope::Scope,
    tasks::Task,
};

use std::fmt::Debug;

use serde::Deserialize;
use serde_yaml;

use crate::{
    environment::{EnvFile, EnvFilesWrapper, RawEnvironment},
    tasks::Tasks,
};

use crate::variables::RawVariables;

use crate::de::deserialize_some;
#[derive(Debug, Deserialize, Clone)]
pub struct GitlabCIConfig {}

fn default_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RawDevrcfile {
    #[serde(default = "default_version")]
    version: String,

    #[serde(default)]
    pub environment: RawEnvironment<String>,

    #[serde(default)]
    pub variables: RawVariables,

    #[serde(default)]
    #[serde(rename(deserialize = "include"))]
    include: IncludeFilesWrapper,

    // #[serde(default)]
    #[serde(rename(deserialize = "env_file"))]
    pub envs_files: Option<EnvFile>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub after_script: Option<Option<Task>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub before_script: Option<Option<Task>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub before_task: Option<Option<Task>>,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub after_task: Option<Option<Task>>,

    #[serde(default)]
    #[serde(rename(deserialize = "devrc_config"))]
    pub config: RawConfig,

    #[serde(flatten)]
    pub tasks: Tasks,

    pub path: Option<PathBuf>,
}

impl RawDevrcfile {
    // TODO: split to separate functions
    pub fn from_file(file: &PathBuf) -> DevrcResult<Self> {
        if !file.exists() {
            return Err(DevrcError::NotExists);
        }

        let contents = match fs::read_to_string(&file) {
            Ok(value) => value,
            Err(error) => return Err(DevrcError::IoError(error)),
        };

        Self::from_str(&contents)
    }

    pub fn get_tasks(&self) -> &Tasks {
        &self.tasks
    }

    pub fn from_str(content: &str) -> DevrcResult<Self> {
        let config: Self = match serde_yaml::from_str(content) {
            Ok(value) => value,
            Err(error) => return Err(DevrcError::YamlParseError(error)),
        };
        Ok(config)
    }

    pub fn get_evolved_scope(&self, parent_scope: Option<Scope>) -> DevrcResult<Scope> {
        let mut scope = Scope::default();
        match parent_scope {
            Some(parent_scope) => self.variables.evaluate(&parent_scope),
            None => self.variables.evaluate(&Scope::default()),
        };

        Ok(scope)
    }

    pub fn setup_path(&mut self, path: PathBuf) -> DevrcResult<()> {
        match fs::canonicalize(path) {
            Ok(value) => self.path = Some(value),
            Err(error) => return Err(DevrcError::IoError(error)),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RawDevrcfile;

    #[test]
    fn test_rawdevrcfile() {
        let devrcfile = RawDevrcfile::default();
        dbg!(devrcfile);
    }
}
