use std::{io::Cursor, path::PathBuf};
use structopt::clap::AppSettings;
use structopt::clap::Shell;
use structopt::StructOpt;

pub fn get_crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(StructOpt, Debug)]
#[structopt(version = get_crate_version())]
#[structopt(name = "Tasks automation tool for developers")]
#[structopt(about = "I am a program and I work, just pass `-h`")]
#[structopt(setting = AppSettings::ColoredHelp)]
/// Commands help
pub struct CommandLine {
    /** Devrc files to use */
    #[structopt(
        parse(from_os_str),
        name = "CONFIG",
        short = "f",
        long = "f",
        verbatim_doc_comment
    )]
    pub configs: Vec<PathBuf>,

    // /** Directory where devrc files located */
    // #[structopt(parse(from_os_str), name = "DIR", short="d", long="dir", verbatim_doc_comment)]
    // pub dirs: Vec<PathBuf>,
    #[structopt(short = "l", long = "list", help = "List option help")]
    pub list: bool,

    /// Read stdin instead of reading default devrcfile
    #[structopt(long = "stdin")]
    pub read_stdin: bool,

    /** Show devrc file variables */
    #[structopt(long = "vars")]
    pub list_vars: bool,

    /// Print shell completion script for <SHELL>
    #[structopt(long, name="SHELL", possible_values = &Shell::variants(), case_insensitive = true)]
    pub completions: Option<Shell>,

    #[structopt(name = "TASKS OR ARGS")]
    // //#[structopt(name = "TASKS", raw(setting = "AppSettings::AllowLeadingHyphen"))]
    pub rest: Vec<String>,

    /// Force to use global .devrc file
    #[structopt(short = "g")]
    pub global: bool,

    /// Dry run mode
    #[structopt(long = "--dry-run")]
    pub dry_run: bool,

    /// Describe task or variable
    #[structopt(short = "d", long = "--describe")]
    pub describe: bool,

    #[structopt(long = "--dbg-runner", hidden = true)]
    pub dbg: bool, // #[structopt(subcommand)]
                   // pub sub: Option<Subcommands>, // /// Trailing newline behavior for the password. If "auto",
                   //                               // /// a trailing newline will be printed iff stdout is detected to be a tty.
                   //                               // #[structopt(long="newline", default_value="auto", raw(possible_values="&NewlineBehavior::variants()"))]
                   //                               // newline: NewlineBehavior
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

// pu fn parse_config(file: PathBuf) -> std::io::Result<Value> {
//     let contents = fs::read_to_string(fs::canonicalize(file)?)?;

//     Ok(contents.parse::<Value>().unwrap())
// }
