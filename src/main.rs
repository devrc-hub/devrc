use std::error::Error;

use devrc::{cli::{self, CommandLine}, raw_devrcfile::RawDevrcfile, utils::{get_local_devrc_file, is_local_devrc_file_exists}};
use devrc::runner::Runner;
use log::info;



fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // info!("devrc tasks automation");

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

    if opt.configs.len() > 0 {
        runner.add_files(opt.configs.clone().as_slice().as_ref())?;
    }

    if opt.dry_run {

        runner.dry_run();
    }

    runner.load()?;

    if opt.list {
        runner.list_tasks()?;
    }
    else if opt.describe {
        runner.describe(opt.rest)?;
    }
    else if opt.dbg {
        runner.diagnostic(opt.rest);
    }
    else {
        runner.run(opt.rest)?;
    }

    // dbg!(runner);

    //let args: Vec<String> = env::args().collect::<Vec<String>>();
    //dbg!(args);

    // let config = match utils::get_config(&opt.input) {
    //     Ok(value) => value,
    //     _ => panic!("Invalid config file: {:?}", &opt.input),
    // };

    Ok(())
}
