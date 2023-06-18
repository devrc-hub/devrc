use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sha256::digest;
use url::Url;

use crate::{errors::DevrcResult, loader::LoadingConfig};

const DEVRC_CACHE_DIR_NAME: &str = "devrc";

/// Get absolute path to devrc cache dir
pub fn get_cache_path() -> Option<PathBuf> {
    dirs_next::cache_dir().map(|path| Path::new(&path).join(DEVRC_CACHE_DIR_NAME))
}

pub fn get_file_cache_meta(url: &Url) -> Option<PathBuf> {
    let hash = digest(url.as_str());
    get_cache_path().map(|path| path.join(format!("{:}.cache", hash)))
}

#[derive(Debug, Default)]
pub struct Cache {}

pub fn save(url: &Url, content: &str) -> DevrcResult<()> {
    if let Some(file) = get_file_cache_meta(url) {
        if let Some(dir) = file.parent() {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
        }

        let mut f = File::create(file)?;
        f.write_all(content.as_bytes())?;
    }
    Ok(())
}

pub fn load(
    url: &Url,
    _loading_config: &LoadingConfig,
    _checksum: Option<&str>,
    ttl: &Duration,
) -> Option<String> {
    if let Some(file) = get_file_cache_meta(url) {
        if !file.exists() {
            return None;
        }

        if let Ok(modified) = fs::metadata(&file).ok()?.modified() {
            let now_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
            let created_timestamp = modified.duration_since(UNIX_EPOCH).ok()?;

            let duration = now_timestamp - created_timestamp;

            if duration < *ttl {
                return fs::read_to_string(file).ok();
            }
        }
    }
    None
}
