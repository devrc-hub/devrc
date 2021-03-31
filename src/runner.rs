use std::{env, io::Read, path::PathBuf, str::FromStr};

use crate::{
    devrc_log::LogLevel,
    devrcfile::Devrcfile,
    docs::DocHelper,
    errors::{DevrcError, DevrcResult},
    interrupt::setup_interrupt_handler,
    raw_devrcfile::RawDevrcfile,
    scope::Scope,
    utils,
    utils::{
        get_absolute_path, get_directory_devrc_file, get_global_devrc_file,
        get_local_user_defined_devrc_file,
    },
    variables::RawVariables,
    workshop::Designer,
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

    pub global_loaded: bool,

    pub loaded_files: Vec<PathBuf>,

    pub log_level: Option<LogLevel>,
    pub designer: Designer,
}

impl Runner {
    pub fn new() -> Self {
        let files: Vec<PathBuf> = Vec::new();

        Runner {
            files,
            use_global: false,
            dry_run: false,
            rest: vec![],
            devrc: Devrcfile::default(),
            global_loaded: false,
            loaded_files: vec![],
            log_level: Some(LogLevel::Info),
            designer: Designer::default(),
        }
    }

    pub fn setup_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    pub fn setup_verbosity(&mut self, level: u8, quiet: bool) -> DevrcResult<()> {
        match (quiet, level) {
            (true, _) => self.log_level = Some(LogLevel::Off),
            (_, x) => self.log_level = Some(LogLevel::from(x)),
        }
        Ok(())
    }

    pub fn setup_variables(&mut self, variables: RawVariables) -> DevrcResult<()> {
        self.devrc.add_variables(variables)
    }

    // Try to get devrcfile
    pub fn get_rawdevrc_file<F>(&self, try_file_func: F) -> DevrcResult<RawDevrcfile>
    where
        F: Fn() -> Option<PathBuf>,
    {
        if let Some(value) = try_file_func() {
            match RawDevrcfile::from_file(&value) {
                Ok(mut parsed_file) => {
                    parsed_file.setup_path(value)?;
                    return Ok(parsed_file);
                }
                Err(error) => return Err(error),
            }
        }
        Err(DevrcError::NotExists)
    }

    pub fn load_global(&mut self) -> DevrcResult<()> {
        if let Ok(devrcfile) = self.get_rawdevrc_file(get_global_devrc_file) {
            if let Some(path) = devrcfile.path.clone() {
                self.add_loaded_file(path)?;
            }
            self.devrc.add_raw_devrcfile(devrcfile)?;
        }
        self.global_loaded = true;

        Ok(())
    }

    pub fn load_file<F>(&mut self, try_file_func: F) -> DevrcResult<()>
    where
        F: Fn() -> Option<PathBuf>,
    {
        if let Ok(devrcfile) = self.get_rawdevrc_file(try_file_func) {
            // Load global file if option is enabled before adding current file
            if devrcfile.is_global_enabled() && self.global_loaded {
                self.load_global()?
            }
            if let Some(path) = devrcfile.path.clone() {
                self.add_loaded_file(path)?;
            }
            self.devrc.add_raw_devrcfile(devrcfile)?;
        }

        Ok(())
    }

    pub fn load(&mut self) -> DevrcResult<()> {
        // Try to load global devrcfile if flag is enabled
        if self.use_global {
            self.load_global()?;
        }

        // Load files only if option specified
        if !self.files.is_empty() {
            let mut loaded_files: Vec<PathBuf> = Vec::new();

            for file in &self.files {
                match self.get_rawdevrc_file(|| Some(file.to_path_buf())) {
                    Ok(devrcfile) => {
                        if let Some(path) = devrcfile.path.clone() {
                            loaded_files.push(path);
                        }
                        self.devrc.add_raw_devrcfile(devrcfile)?;
                    }
                    Err(error) => return Err(error),
                }
            }

            for file in loaded_files {
                self.add_loaded_file(file)?;
            }
        } else {
            // Try to load Devrcfile from current directory

            self.load_file(get_directory_devrc_file)?;

            // Try to load Devrcfile.local
            self.load_file(get_local_user_defined_devrc_file)?;
        }

        self.devrc.setup_dry_run(self.dry_run)?;

        if let Some(level) = &self.log_level {
            self.devrc.setup_log_level(level.clone())?;
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
        self.devrc.setup_dry_run(self.dry_run)?;

        if let Some(level) = &self.log_level {
            self.devrc.setup_log_level(level.clone())?;
        }

        Ok(())
    }

    pub fn add_loaded_file(&mut self, file: PathBuf) -> DevrcResult<()> {
        self.loaded_files.push(file);
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

    /// Show detailed tasks list with short descriptions
    pub fn list_tasks_detailed(&self) -> DevrcResult<()> {
        println!("Available tasks:");
        let offset = 2;
        let (max_taskname_width, _) = self.devrc.get_max_taskname_width();
        for (name, task) in self.devrc.get_tasks_docs() {
            let help = task.format_help()?;
            let parameters = task.format_parameters_help(&self.designer)?;

            if name.starts_with('_') {
                continue;
            }

            // TODO: Add colours
            if !help.is_empty() {
                println!("\n{:width$}# {}", "", help.trim_end(), width = offset);
            }

            println!(
                "{:width$}{}{:max_taskname_width$}{} {}",
                "",
                self.designer.task_name().prefix(),
                name,
                self.designer.task_name().suffix(),
                parameters,
                // help,
                width = offset,
                max_taskname_width = max_taskname_width
            );
        }

        Ok(())
    }

    /// Show tasks list with short descriptions
    pub fn list_tasks(&self) -> DevrcResult<()> {
        let offset = 2;
        let (max_taskname_width, _) = self.devrc.get_max_taskname_width();
        for (name, task) in self.devrc.get_tasks_docs() {
            let help = task.format_help()?;

            if name.starts_with('_') {
                continue;
            }

            let help_string = if help.is_empty() {
                "".to_string()
            } else {
                format!("# {}", help)
            };

            let parameters_marker = if task.has_parameters() {
                let color = self.designer.parameter_name();
                format!("{}*{}", color.prefix(), color.suffix())
            } else {
                " ".to_string()
            };

            println!(
                "{:width$}{}{:max_taskname_width$}{} {} {}",
                "",
                self.designer.task_name().prefix(),
                name,
                self.designer.task_name().suffix(),
                parameters_marker,
                help_string,
                // help,
                width = offset,
                max_taskname_width = max_taskname_width
            );
        }

        Ok(())
    }

    /// Show global variables and their computed values
    pub fn list_vars(&self) -> DevrcResult<()> {
        println!("List global devrc variables:");

        let max_variable_name_width = self.devrc.variables.get_max_key_width();

        for (name, value) in self.devrc.get_vars() {
            println!(
                "{:width$}{}{:max_variable_name_width$}{} = \"{}\"",
                "",
                self.designer.variable().prefix(),
                name,
                self.designer.variable().suffix(),
                value,
                width = 2,
                max_variable_name_width = max_variable_name_width
            );
        }
        Ok(())
    }

    /// Show global environment variables and their computed values
    pub fn list_env_vars(&self) -> DevrcResult<()> {
        println!("List global devrc environment variables:");

        let max_variable_name_width = self.devrc.environment.get_max_key_width();

        for (name, value) in self.devrc.get_environment_vars() {
            println!(
                "{:width$}{}{:max_variable_name_width$}{} = \"{}\"",
                "",
                self.designer.evariable().prefix(),
                name,
                self.designer.evariable().suffix(),
                value,
                width = 2,
                max_variable_name_width = max_variable_name_width
            );
        }
        Ok(())
    }

    /// Load global devrc
    pub fn use_global(&mut self) {
        self.use_global = true;
    }

    /// Show description for given task, variable or environment variable
    pub fn describe(&self, params: Vec<String>) -> DevrcResult<()> {
        for name in params {
            let task = self.devrc.find_task(&name)?;
            println!(
                "Usage: {}\nDescription: {}",
                task.get_usage_help(&name, &self.designer)?,
                task.format_help()?,
                // width = 2,
            );

            if let Some(example) = task.get_example() {
                println!("Examples: \n{}", example);
            }
            println!();
        }
        Ok(())
    }

    pub fn get_calculated_scope(&self, _scope: &Scope) {}

    pub fn diagnostic(&mut self, params: Vec<String>) {
        println!("Show diagnostic info:");

        self.rest = params;

        info!(
            "Global defined interpreter: `{:}`",
            &self.devrc.config.interpreter
        );

        if let Some(value) = get_global_devrc_file() {
            info!("Global .devrc exists: {:?}", value);
        }

        if let Some(value) = get_directory_devrc_file() {
            info!("Local directory Devrcfile exists: {:?}", value);
        }

        if let Some(value) = get_local_user_defined_devrc_file() {
            info!("Local user defined Devrcfile.local exists: {:?}", value);
        }

        for file in &self.files {
            if let Ok(file) = get_absolute_path(&file, None) {
                if file.exists() {
                    info!("Given Devrcfile exists: {:?}", file);
                } else {
                    info!("Given Devrcfile not exists: {:?}", file);
                }
            } else {
                error!("Given Devrcfile with broken path: {:?}", &file);
            }
        }

        for file in &self.loaded_files {
            info!("Loaded file: {:?}", file);
        }

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
