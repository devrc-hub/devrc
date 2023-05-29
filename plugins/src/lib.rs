#![deny(clippy::all)]
// List of ignored linters
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::from_over_into)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::borrowed_box)]

extern crate libloading;

pub mod config;
pub mod errors;
pub mod execution;
pub mod options;
pub mod plugin;

pub use errors::{DevrcPluginError, DevrcPluginResult};
pub use execution::{ExecutionPlugin, ExecutionPluginManager};
pub use plugin::Plugin;
