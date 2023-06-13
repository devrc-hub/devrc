use std::{
    env,
    path::{Path, PathBuf},
};

/// netrc file name
const NETRC_FILE_NAME: &str = ".netrc";

/// Get absolute path to user's netrc file
pub fn get_user_defined_netrc_path() -> Option<PathBuf> {
    match env::var("NETRC") {
        Ok(path) => Some(PathBuf::from(path)),
        Err(_) => dirs_next::home_dir().map(|path| Path::new(&path).join(NETRC_FILE_NAME)),
    }
}
