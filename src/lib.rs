// List of ignored linters
#![allow()]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::correctness
)]
#![feature(custom_inner_attributes)]
#![clippy::msrv = "1.48.0"]

#[macro_use]
extern crate log;

pub mod cli;
pub mod common;
pub mod config;
pub mod de;
pub mod devrcfile;
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
pub mod utils;
pub mod variables;
