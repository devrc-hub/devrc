use std::error::Error;

use devrc::{
    cli::{self, CommandLine},
    runner::Runner,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // logger::init(&LoggerOptions {
    //     level: cli_args.log_level.clone(),
    //     color: !cli_args.disable_color,
    // });

    let opt = cli::parse_args();

    let mut runner = Runner::new();

    // TODO: add devrcfile tasks to completions
    if let Some(shell) = opt.completions {
        CommandLine::generate_completions(shell);
        return Ok(());
    }

    if opt.global {
        runner.use_global();
    }

    if !opt.configs.is_empty() {
        runner.add_files(opt.configs.as_slice().as_ref())?;
    }

    if opt.dry_run {
        runner.setup_dry_run();
    }

    if opt.read_stdin {
        runner.load_stdin()?
    } else {
        runner.load()?;
    }

    if opt.list {
        runner.list_tasks()?;
    } else if opt.list_vars {
        runner.list_vars()?;
    } else if opt.list_env_vars {
        runner.list_env_vars()?;
    } else if opt.describe {
        runner.describe(opt.rest)?;
    } else if opt.dbg {
        runner.diagnostic(opt.rest);
    } else {
        runner.run(opt.rest)?;
    }

    Ok(())
}
