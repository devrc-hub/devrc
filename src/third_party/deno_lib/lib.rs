#![deny(warnings)]
//#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::from_over_into)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub(crate) mod ast;
pub(crate) mod auth_tokens;
pub(crate) mod checksum;
pub mod colors;
pub(crate) mod deno_dir;
pub(crate) mod diagnostics;
// pub(crate) mod diff;
pub(crate) mod disk_cache;
pub mod errors;
pub mod file_fetcher;
// pub(crate) mod file_watcher;
pub mod flags;
pub mod flags_allow_net;
pub mod fmt_errors;
pub(crate) mod fs_util;
pub(crate) mod http_cache;
pub(crate) mod http_util;
pub(crate) mod import_map;
pub(crate) mod info;
pub(crate) mod lockfile;
pub mod media_type;
pub(crate) mod module_graph;
pub mod module_loader;
pub mod ops;
pub mod program_state;
pub mod source_maps;
pub(crate) mod specifier_handler;
// pub(crate) mod standalone;
pub(crate) mod text_encoding;
// pub(crate) mod tokio_util;
// pub(crate) mod tools;
pub(crate) mod tsc;
pub(crate) mod tsc_config;
pub mod version;
pub mod workers;

use deno_core::{error::AnyError, serde_json};
use std::io::Write;

pub fn get_types(unstable: bool) -> String {
    let mut types = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        crate::tsc::DENO_NS_LIB,
        crate::tsc::DENO_WEB_LIB,
        crate::tsc::DENO_FETCH_LIB,
        crate::tsc::DENO_WEBGPU_LIB,
        crate::tsc::DENO_WEBSOCKET_LIB,
        crate::tsc::SHARED_GLOBALS_LIB,
        crate::tsc::WINDOW_LIB,
    );

    if unstable {
        types.push_str(&format!("\n{}", crate::tsc::UNSTABLE_NS_LIB,));
    }

    types
}

pub fn write_to_stdout_ignore_sigpipe(bytes: &[u8]) -> Result<(), std::io::Error> {
    use std::io::ErrorKind;

    match std::io::stdout().write_all(bytes) {
        Ok(()) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::BrokenPipe => Ok(()),
            _ => Err(e),
        },
    }
}

pub fn write_json_to_stdout<T>(value: &T) -> Result<(), AnyError>
where
    T: ?Sized + serde::ser::Serialize,
{
    let writer = std::io::BufWriter::new(std::io::stdout());
    serde_json::to_writer_pretty(writer, value).map_err(AnyError::from)
}
