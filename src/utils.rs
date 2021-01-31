use std::{env, fs};

use std::path::Path;
use std::path::PathBuf;

use dirs;

use crate::errors::{DevrcError, DevrcResult};

/// Current directory/project devrc file name
const DEFAULT_DEVRC_FILE_NAME: &str = "devrc";

/// Global devrc file name
const HOME_DEVRC_FILE_NAME: &str = ".devrc";

/// User defined local devrc file
/// that override project defined tasks and variables
const LOCAL_DEVRC_FILE_NAME: &str = "devrc.local";


pub fn get_env_dict() {
    // We will iterate through the references to the element returned by
    // env::vars();
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
}

pub fn get_local_file_content() {
    dbg!("Fetch content from local file");
}

// pub fn parse_config(file: PathBuf) -> std::io::Result<Value> {
//     let contents = fs::read_to_string(fs::canonicalize(file)?)?;

//     Ok(contents.parse::<Value>().unwrap())
// }

pub fn expand_path(file: &PathBuf) -> DevrcResult<PathBuf> {
    match fs::canonicalize(file){
        Ok(value) => Ok(value),
        Err(error) => Err(DevrcError::IoError(error))
    }
}

pub fn get_devrc_file_name() -> String {
    match env::var("DEVRC_FILE") {
        Ok(val) => {
            debug!("DERVC_FILE environment variable exists: {:?}", val);
            val.into()
        }
        _ => {
            DEFAULT_DEVRC_FILE_NAME.into()
        }
    }
}

pub fn get_local_devrc_file() -> Option<PathBuf> {
    match env::current_dir() {
        Ok(path) => Some(Path::new(&path).join(get_devrc_file_name()).to_path_buf()),
        Err(_) => None,
    }
}

pub fn get_global_devrc_file() -> Option<PathBuf> {
    match dirs::home_dir() {
        Some(path) => Some(Path::new(&path).join(HOME_DEVRC_FILE_NAME).to_path_buf()),
        _ => None,
    }
}

pub fn is_local_devrc_file_exists() -> bool {
    match get_local_devrc_file() {
        Some(path) => path.exists(),
        _ => return false,
    }
}

pub fn is_global_devrc_file_exists() -> bool {
    match get_global_devrc_file() {
        Some(path) => path.exists(),
        _ => return false,
    }
}
