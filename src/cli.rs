use std::{io::Cursor, path::PathBuf};
use structopt::{
    clap::{AppSettings, Shell},
    StructOpt,
};

use std::error::Error;

use crate::errors::{DevrcError, DevrcResult};

pub fn get_crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Parse a single key-value pair
fn parse_key_val<T>(s: &str) -> DevrcResult<(T, T)>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    let pos = s.find('=').ok_or({
        //format!("invalid KEY=value: no `=` found in `{}`", s)}
        DevrcError::InvalidArgument
    })?;
    Ok((
        s[..pos].parse().map_err(|_| DevrcError::InvalidArgument)?,
        s[pos + 1..]
            .parse()
            .map_err(|_| DevrcError::InvalidArgument)?,
    ))
}

#[derive(StructOpt, Debug)]
#[structopt(version = get_crate_version())]
#[structopt(name = "devrc")]
#[structopt(about = "⚡ task automation tool on steroids ⚡")]
#[structopt(after_help = "Wish you productive coding!
Visit https://github.com/devrc-hub/devrc to get more info about devrc.")]
#[structopt(setting = AppSettings::ColoredHelp)]
pub struct CommandLine {
    /// Specify an alternate Devrcfile files
    /// (default: Devrcfile)
    #[structopt(
        parse(from_os_str),
        name = "CONFIG",
        short = "f",
        long = "f",
        verbatim_doc_comment
    )]
    pub configs: Vec<PathBuf>,

    /// List available tasks and their arguments
    #[structopt(short = "l", long = "list")]
    pub list: bool,

    /// Show detailed tasks list
    #[structopt(short = "d", long = "detailed")]
    pub detailed: bool,

    /// Read stdin instead of reading default devrcfile
    #[structopt(long = "stdin")]
    pub read_stdin: bool,

    /// Show global variables
    #[structopt(long = "variables")]
    pub list_vars: bool,

    /// Show global environment variables
    #[structopt(long = "evariables")]
    pub list_env_vars: bool,

    /// Print shell completion script for <SHELL>
    #[structopt(long, name="SHELL", possible_values = &Shell::variants(), case_insensitive = true)]
    pub completions: Option<Shell>,

    #[structopt(name = "TASKS OR ARGS")]
    // //#[structopt(name = "TASKS", raw(setting = "AppSettings::AllowLeadingHyphen"))]
    pub rest: Vec<String>,

    /// Force to use global .devrc file
    #[structopt(short = "g")]
    pub global: bool,

    /// Print commands and skip execution
    #[structopt(long = "--dry-run")]
    pub dry_run: bool,

    /// Describe task
    #[structopt(long = "--describe")]
    pub describe: bool,

    /// Show debug info
    #[structopt(long = "--dbg-runner", hidden = true)]
    pub dbg: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Enable verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Suppress all output
    #[structopt(short, long)]
    pub quiet: bool,

    /// Override <VARIABLE> with <VALUE>
    #[structopt(long = "--set", parse(try_from_str = parse_key_val), name="VAR=VALUE")]
    pub set: Vec<(String, String)>,
}

impl CommandLine {
    pub fn generate_completions(shell: Shell) {
        let mut cursor = Cursor::new(Vec::new());
        Self::clap().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut cursor);
        println!(
            "{}",
            String::from_utf8(cursor.into_inner())
                .expect("Clap completion not UTF-8")
                .trim()
        );
    }
}

#[derive(Debug, PartialEq, StructOpt)]
pub enum Subcommands {
    // `external_subcommand` tells structopt to put
    // all the extra arguments into this Vec
    #[structopt(external_subcommand)]
    Other(Vec<String>),
}

pub fn parse_args() -> CommandLine {
    CommandLine::from_args()
}
