use std::{env, fs};

use std::path::{Path, PathBuf};

use crate::errors::{DevrcError, DevrcResult};

/// Current directory/project devrc file name
const DEFAULT_DEVRC_FILE_NAME: &str = "Devrcfile";

/// Global devrc file name
const HOME_DEVRC_FILE_NAME: &str = ".devrc";

/// User defined local devrcfile
/// It overwrite project defined tasks and variables
#[allow(dead_code)]
const LOCAL_DEVRC_FILE_NAME: &str = "Devrcfile.local";

pub fn get_env_dict() {
    // We will iterate through the references to the element returned by
    // env::vars();
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
}

pub fn get_absolute_path(file: &PathBuf, base: Option<&PathBuf>) -> DevrcResult<PathBuf> {
    if file.is_absolute() {
        return Ok(file.to_path_buf());
    }

    let file = if let Some(value) = base {
        let mut new_path = value.clone();
        if new_path.is_file() {
            if let Some(value) = new_path.parent() {
                new_path = value.to_path_buf();
            }
        }
        new_path.push(file);
        new_path
    } else {
        file.clone()
    };

    match fs::canonicalize(file) {
        Ok(value) => Ok(value),
        Err(error) => Err(DevrcError::IoError(error)),
    }
}

pub fn get_devrc_file_name() -> String {
    match env::var("DEVRC_FILE") {
        Ok(val) => {
            debug!("DERVC_FILE environment variable exists: {:?}", val);
            val
        }
        _ => DEFAULT_DEVRC_FILE_NAME.into(),
    }
}

pub fn get_directory_devrc_file() -> Option<PathBuf> {
    match env::current_dir() {
        Ok(path) => Some(Path::new(&path).join(get_devrc_file_name())),
        Err(_) => None,
    }
}

pub fn get_global_devrc_file() -> Option<PathBuf> {
    match dirs_next::home_dir() {
        Some(path) => Some(Path::new(&path).join(HOME_DEVRC_FILE_NAME)),
        _ => None,
    }
}

pub fn get_local_user_defined_devrc_file() -> Option<PathBuf> {
    match env::current_dir() {
        Ok(path) => Some(Path::new(&path).join(LOCAL_DEVRC_FILE_NAME)),
        Err(_) => None,
    }
}

pub fn is_local_devrc_file_exists() -> bool {
    match get_directory_devrc_file() {
        Some(path) => path.exists(),
        _ => false,
    }
}

pub fn is_global_devrc_file_exists() -> bool {
    match get_global_devrc_file() {
        Some(path) => path.exists(),
        _ => false,
    }
}
