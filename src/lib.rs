#![deny(
    clippy::all,
    // clippy::pedantic,
    // clippy::restriction,
    // clippy::correctness
)]
// List of ignored linters
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::from_over_into)]
#![allow(clippy::ptr_arg)]

// #![feature(custom_inner_attributes)]
// #![clippy::msrv = "1.48.0"]

#[macro_use]
extern crate log;

pub mod cli;
pub mod common;
pub mod config;
pub mod de;

#[cfg(feature = "deno")]
pub mod denoland;
pub mod devrc_log;
pub mod devrcfile;
pub mod docs;
pub mod environment;
pub mod errors;
pub mod evaluate;
pub mod execute;
pub mod include;
pub mod interpreter;
pub mod interrupt;
pub mod raw_devrcfile;
pub mod runner;
pub mod scope;
pub mod tasks;
pub mod template;
pub mod user_agent;
pub mod utils;
pub mod variables;
pub mod version;
pub mod workshop;
