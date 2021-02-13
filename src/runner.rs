use std::{env, io::Read, path::PathBuf, str::FromStr};

use crate::{
    devrcfile::Devrcfile,
    errors::{DevrcError, DevrcResult},
    interrupt::setup_interrupt_handler,
    raw_devrcfile::RawDevrcfile,
    scope::Scope,
    utils,
    utils::{get_absolute_path, get_global_devrc_file, get_local_devrc_file},
};

use std::fmt::Debug;

use std::io;

#[derive(Debug, Clone)]
pub struct Runner {
    pub files: Vec<PathBuf>,
    use_global: bool,
    dry_run: bool,
    rest: Vec<String>,

    /// Assembled tasks library
    pub devrc: Devrcfile,
}

impl Runner {
    pub fn new() -> Self {
        let files: Vec<PathBuf> = Vec::new();

        Runner {
            files,
            use_global: false,
            rest: vec![],
            dry_run: false,
            devrc: Devrcfile::default(),
        }
    }

    pub fn dry_run(&mut self) {
        self.dry_run = true;
    }

    pub fn get_global_rawdevrc_file(&self) -> DevrcResult<RawDevrcfile> {
        if let Some(value) = get_global_devrc_file() {
            match RawDevrcfile::from_file(&value) {
                Ok(mut parsed_file) => {
                    parsed_file.setup_path(value)?;
                    return Ok(parsed_file);
                }
                Err(error) => return Err(error),
            }
        }
        Err(DevrcError::GlobalNotExists)
    }

    pub fn get_local_rawdevrc_file(&self) -> DevrcResult<RawDevrcfile> {
        if let Some(value) = get_local_devrc_file() {
            match RawDevrcfile::from_file(&value) {
                Ok(mut parsed_file) => {
                    parsed_file.setup_path(value)?;
                    return Ok(parsed_file);
                }
                Err(error) => return Err(error),
            }
        }
        Err(DevrcError::LocalNotExists)
    }

    pub fn load(&mut self) -> DevrcResult<()> {
        if let Ok(devrcfile) = self.get_global_rawdevrc_file() {
            self.devrc.add_raw_devrcfile(devrcfile)?;
        }

        for file in self.files.iter() {
            match RawDevrcfile::from_file(file) {
                Ok(parsed_file) => {
                    let mut parsed_file: RawDevrcfile = parsed_file;
                    parsed_file.setup_path(file.to_path_buf())?;
                    dbg!(&parsed_file);
                    self.devrc.add_raw_devrcfile(parsed_file)?;
                }
                Err(error) => return Err(error),
            };
        }

        if let Ok(devrcfile) = self.get_local_rawdevrc_file() {
            self.devrc.add_raw_devrcfile(devrcfile)?;
        }

        Ok(())
    }

    pub fn load_stdin(&mut self) -> DevrcResult<()> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        match RawDevrcfile::from_str(&buffer) {
            Ok(parsed_file) => {
                let mut parsed_file: RawDevrcfile = parsed_file;
                parsed_file.setup_path(PathBuf::from("/dev/stdin"))?;
                self.devrc.add_raw_devrcfile(parsed_file)?;
            }
            Err(error) => return Err(error),
        }
        Ok(())
    }

    pub fn add_file(&mut self, file: PathBuf) -> DevrcResult<()> {
        match utils::get_absolute_path(&file, env::current_dir().ok().as_ref()) {
            Ok(path) => {
                self.files.push(path);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn add_files(&mut self, files: &[PathBuf]) -> DevrcResult<()> {
        for file in files.iter() {
            self.add_file(file.to_path_buf())?;
        }
        Ok(())
    }

    /// Execute given commands
    pub fn run(&mut self, params: Vec<String>) -> DevrcResult<()> {
        self.rest = params;
        setup_interrupt_handler();
        self.devrc.run(&self.rest)
    }

    /// Show tasks list with short descriptions
    pub fn list_tasks(&self) -> DevrcResult<()> {
        println!("Available devrc tasks:");

        // TODO: remove copy
        let (max_taskname_width, _) = self.devrc.get_max_taskname_width();
        for (name, task) in self.devrc.tasks.items.clone() {
            let help = task.format_help()?;

            if name.starts_with('_') {
                continue;
            }

            // TODO: Add colours
            println!(
                "{:width$}{:max_taskname_width$}  {}",
                "",
                name,
                help,
                width = 2,
                max_taskname_width = max_taskname_width
            );
        }

        Ok(())
    }

    pub fn list_vars(&self) -> DevrcResult<()> {
        println!("List devrc variables:");
        Ok(())
    }

    /// Load global devrc
    pub fn use_global(&mut self) {
        println!("Use global devrc file");
        self.use_global = true;
    }

    //  else if utils::is_global_devrc_file_exists() {
    //     config.add_file(utils::get_global_devrc_file().unwrap());
    // } else {
    //     println!("Show help")
    // }

    /// Show description for given task, variable or environment variable
    pub fn describe(&self, _params: Vec<String>) -> DevrcResult<()> {
        println!("Describe task or variable");
        Ok(())
    }

    pub fn get_calculated_scope(&self, _scope: &Scope) {}

    pub fn diagnostic(&mut self, params: Vec<String>) {
        println!("Show diagnostic info:");

        self.rest = params;

        if let Some(value) = get_global_devrc_file() {
            info!("Global devrcfile exists: {:?}", value);
        }

        if let Some(value) = get_local_devrc_file() {
            info!("Local devrcfile exists: {:?}", value);
        }

        for file in &self.files {
            if let Ok(file) = get_absolute_path(&file, None) {
                if file.exists() {
                    info!("Given devrcfile exists: {:?}", file);
                } else {
                    info!("Given devrcfile not exists: {:?}", file);
                }
            } else {
                error!("Given devrcfile with broken path: {:?}", &file);
            }
        }

        info!(
            "Global defined interpreter: `{:}`",
            &self.devrc.config.interpreter
        );

        dbg!(self);
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

// pub fn get_config<T>(file: &PathBuf) -> DevrcResult<T>
// where T: for<'de> Deserialize<'de>
// {
//     let contents = match fs::read_to_string(&file) {
//         Ok(value) => value,
//         Err(error) => {
//             panic!("Can't read config file: {:?}", &file);
//             return Err(DevrcError::IoError(error))
//         },
//     };

//     let config: T = match serde_yaml::from_str(&contents) {
//         Ok(value) => value,
//         Err(error) => return Err(DevrcError::YamlParseError(error))
//     };

//     Ok(config)
// }

#[cfg(test)]
mod tests {

    #[test]
    fn test_devrcfile() {}
}
