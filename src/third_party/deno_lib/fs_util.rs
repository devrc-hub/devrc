// Copyright 2018-2021 the Deno authors. All rights reserved. MIT license.

pub use deno_core::normalize_path;
use deno_runtime::deno_crypto::rand;

use std::{
    fs::OpenOptions,
    io::{Error, Write},
    path::{Path, PathBuf},
};

pub fn atomic_write_file<T: AsRef<[u8]>>(
    filename: &Path,
    data: T,
    mode: u32,
) -> std::io::Result<()> {
    let rand: String = (0..4)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect();
    let extension = format!("{}.tmp", rand);
    let tmp_file = filename.with_extension(extension);
    write_file(&tmp_file, data, mode)?;
    std::fs::rename(tmp_file, filename)?;
    Ok(())
}

pub fn write_file<T: AsRef<[u8]>>(filename: &Path, data: T, mode: u32) -> std::io::Result<()> {
    write_file_2(filename, data, true, mode, true, false)
}

pub fn write_file_2<T: AsRef<[u8]>>(
    filename: &Path,
    data: T,
    update_mode: bool,
    mode: u32,
    is_create: bool,
    is_append: bool,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(false)
        .write(true)
        .append(is_append)
        .truncate(!is_append)
        .create(is_create)
        .open(filename)?;

    if update_mode {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = mode & 0o777;
            let permissions = PermissionsExt::from_mode(mode);
            file.set_permissions(permissions)?;
        }
        #[cfg(not(unix))]
        let _ = mode;
    }

    file.write_all(data.as_ref())
}

/// Similar to `std::fs::canonicalize()` but strips UNC prefixes on Windows.
pub fn canonicalize_path(path: &Path) -> Result<PathBuf, Error> {
    let mut canonicalized_path = path.canonicalize()?;
    if cfg!(windows) {
        canonicalized_path = PathBuf::from(
            canonicalized_path
                .display()
                .to_string()
                .trim_start_matches("\\\\?\\"),
        );
    }
    Ok(canonicalized_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_from_cwd_child() {
        let cwd = current_dir().unwrap();
        assert_eq!(resolve_from_cwd(Path::new("a")).unwrap(), cwd.join("a"));
    }

    #[test]
    fn resolve_from_cwd_dot() {
        let cwd = current_dir().unwrap();
        assert_eq!(resolve_from_cwd(Path::new(".")).unwrap(), cwd);
    }

    #[test]
    fn resolve_from_cwd_parent() {
        let cwd = current_dir().unwrap();
        assert_eq!(resolve_from_cwd(Path::new("a/..")).unwrap(), cwd);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("a/../b")), PathBuf::from("b"));
        assert_eq!(normalize_path(Path::new("a/./b/")), PathBuf::from("a/b/"));
        assert_eq!(
            normalize_path(Path::new("a/./b/../c")),
            PathBuf::from("a/c")
        );

        if cfg!(windows) {
            assert_eq!(
                normalize_path(Path::new("C:\\a\\.\\b\\..\\c")),
                PathBuf::from("C:\\a\\c")
            );
        }
    }

    // TODO: Get a good expected value here for Windows.
    #[cfg(not(windows))]
    #[test]
    fn resolve_from_cwd_absolute() {
        let expected = Path::new("/a");
        assert_eq!(resolve_from_cwd(expected).unwrap(), expected);
    }
}
