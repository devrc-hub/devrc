use std::{cell::RefCell, convert::TryFrom, env, fs, io::Read, path::PathBuf, time::Duration};

use devrc_core::{logging::LogLevel, workshop::Designer};
use indexmap::IndexMap;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use url::Url;

use crate::{
    auth::Auth,
    cache::Cache,
    devrcfile::Devrcfile,
    docs::DocHelper,
    errors::{DevrcError, DevrcResult},
    include::{FileInclude, Include, UrlInclude},
    interrupt::setup_interrupt_handler,
    loader::LoadingConfig,
    raw::devrcfile::{Kind, RawDevrcfile},
    registry::Registry,
    resolver::{Location, PathResolve},
    scope::Scope,
    tasks::arguments::TaskArguments,
    utils,
    utils::{
        get_absolute_path, get_directory_devrc_file, get_global_devrc_file,
        get_local_user_defined_devrc_file,
    },
    variables::RawVariables,
};

use sha256::digest;

use std::{fmt::Debug, rc::Rc};

use std::io;

const DEFAULT_MAX_NESTING_LEVEL: u32 = 10;

use devrc_plugins::execution::ExecutionPluginManager;

#[derive(Debug)]
pub struct Runner {
    pub files: Vec<PathBuf>,
    use_global: bool,
    dry_run: bool,
    rest: Vec<String>,

    /// Assembled tasks library
    pub devrc: Devrcfile,

    pub global_loaded: bool,

    pub loaded_locations: Vec<Location>,

    pub log_level: Option<LogLevel>,
    pub designer: Designer,

    pub global_scope: Rc<RefCell<Scope>>,

    pub registry: Registry,

    pub max_nesting_level: u32,

    pub execution_plugin_registry: Rc<RefCell<ExecutionPluginManager>>,

    pub cache: Cache,
}

impl Runner {
    pub fn new() -> Self {
        let files: Vec<PathBuf> = Vec::new();
        let scope = Rc::new(RefCell::new(Scope {
            name: "devrcfile".to_string(),
            ..Default::default()
        }));
        let plugin_manager = Rc::new(RefCell::new(ExecutionPluginManager::new()));
        let devrcfile = Devrcfile::with_scope(Rc::clone(&scope))
            .with_execution_plugin_manager(Rc::clone(&plugin_manager));

        Runner {
            files,
            use_global: false,
            dry_run: false,
            rest: vec![],
            devrc: devrcfile,
            global_loaded: false,
            loaded_locations: vec![],
            log_level: Some(LogLevel::Info),
            designer: Designer::default(),
            global_scope: scope,
            registry: Registry::default(),
            max_nesting_level: DEFAULT_MAX_NESTING_LEVEL,
            execution_plugin_registry: plugin_manager,
            cache: Cache::default(),
        }
    }

    pub fn setup_dry_run(&mut self, dry_run: bool) -> DevrcResult<()> {
        self.dry_run = dry_run;
        Ok(())
    }

    pub fn setup_verbosity(&mut self, level: u8, quiet: bool) -> DevrcResult<()> {
        match (quiet, level) {
            (true, _) => self.log_level = Some(LogLevel::Off),
            (_, x) => self.log_level = Some(LogLevel::from(x)),
        }
        Ok(())
    }

    pub fn setup_variables(&mut self, variables: RawVariables) -> DevrcResult<()> {
        self.devrc.process_variables(variables)
    }

    pub fn get_logger(&self) -> LogLevel {
        if let Some(level) = self.log_level.clone() {
            return level;
        }
        self.devrc.config.log_level.clone()
    }

    pub fn get_cache_ttl(&self) -> Option<Duration> {
        self.devrc.config.cache_ttl
    }

    pub fn load(&mut self) -> DevrcResult<()> {
        if let Some(level) = &self.log_level {
            self.devrc.setup_log_level(level.clone())?;
        }

        // Try to load global devrcfile if flag is enabled
        if self.use_global {
            self.load_global()?;
        }

        let loading_config = LoadingConfig::default()
            .with_log_level(self.get_logger())
            .with_cache_ttl(self.get_cache_ttl());

        // Load files only if option specified
        if !self.files.is_empty() {
            let files = self.files.clone();

            for file in files {
                self.load_file(file.to_path_buf(), loading_config.clone(), Kind::Args)?;
            }
        } else {
            if let Some(file) = get_directory_devrc_file() {
                if file.exists() {
                    // Try to load Devrcfile from current directory
                    self.load_file(file, loading_config.clone(), Kind::Directory)?;
                }
            }

            if let Some(file) = get_local_user_defined_devrc_file() {
                if file.exists() {
                    // Try to load Devrcfile.local
                    self.load_file(file, loading_config, Kind::DirectoryLocal)?;
                }
            }
        }

        self.devrc.setup_dry_run(self.dry_run)?;

        if let Some(level) = &self.log_level {
            self.devrc.setup_log_level(level.clone())?;
        }

        // Init plugins
        self.load_plugins()?;

        Ok(())
    }

    pub fn load_plugins(&mut self) -> DevrcResult<()> {
        let mut plugins_registry = (*self.execution_plugin_registry)
            .try_borrow_mut()
            .map_err(|_| DevrcError::RuntimeError)?;

        plugins_registry.setup_logger(self.get_logger());

        for (name, path) in self.devrc.config.plugins.clone().into_iter() {
            let abs_path = utils::get_absolute_path(&path, None)?;
            unsafe {
                if !abs_path.exists() {
                    return Err(DevrcError::PluginFileNotExists(abs_path));
                }
                plugins_registry.load_plugin(&name, abs_path.as_os_str(), self.get_logger())?;
            }
        }
        Ok(())
    }

    pub fn load_global(&mut self) -> DevrcResult<()> {
        if let Some(file) = get_global_devrc_file() {
            self.get_logger().debug(
                &format!("\n==> Loading GLOBAL: `{}` ...", &file.display()),
                &self.designer.banner(),
            );

            if file.exists() {
                let content = fs::read_to_string(&file).map_err(DevrcError::IoError)?;
                let location = Location::LocalFile(file);
                let loading_config = LoadingConfig::default().with_log_level(self.get_logger());

                self.load_from_str(&content, location, Kind::Global, loading_config)?;
                self.global_loaded = true;
            } else {
                return Err(DevrcError::FileNotExists(file));
            }
        }
        Ok(())
    }

    pub fn load_file(
        &mut self,
        file: PathBuf,
        loading_config: LoadingConfig,
        kind: Kind,
    ) -> DevrcResult<()> {
        self.get_logger().debug(
            &format!("\n==> Loading FILE: `{}` ...", &file.display()),
            &self.designer.banner(),
        );

        if loading_config.level > self.max_nesting_level {
            return Err(DevrcError::NestingLevelExceed);
        }

        if file.exists() {
            let content = fs::read_to_string(&file).map_err(DevrcError::IoError)?;
            let location = Location::LocalFile(file);
            self.load_from_str(&content, location, kind, loading_config)?;
        } else {
            return Err(DevrcError::FileNotExists(file));
        }

        Ok(())
    }

    pub fn load_stdin(&mut self) -> DevrcResult<()> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let loading_config = LoadingConfig::default()
            .with_log_level(self.get_logger())
            .with_cache_ttl(self.get_cache_ttl());
        self.load_from_str(&buffer, Location::StdIn, Kind::StdIn, loading_config)
    }

    pub fn load_from_str(
        &mut self,
        input: &str,
        location: Location,
        kind: Kind,
        loading_config: LoadingConfig,
    ) -> DevrcResult<()> {
        match RawDevrcfile::prepared_from_str(
            input,
            location.clone(),
            kind.clone(),
            loading_config.clone(),
        ) {
            Ok(raw_devrcfile) => {
                if matches!(&kind, Kind::Directory | Kind::DirectoryLocal | Kind::StdIn)
                    && raw_devrcfile.is_global_enabled()
                    && !self.global_loaded
                {
                    self.load_global()?
                }

                self.devrc.add_config(raw_devrcfile.config.clone(), &kind)?;

                self.load_include(
                    raw_devrcfile.clone(),
                    location,
                    loading_config.with_cache_ttl(self.get_cache_ttl()),
                )?;

                self.devrc.add_raw_devrcfile(raw_devrcfile.clone(), &kind)?;
                self.registry.add(raw_devrcfile)?;
            }
            Err(error) => return Err(error),
        }
        self.devrc.setup_dry_run(self.dry_run)?;

        if let Some(level) = &self.log_level {
            self.devrc.setup_log_level(level.clone())?;
        }
        Ok(())
    }

    pub fn load_url(
        &mut self,
        url: Url,
        loading_config: LoadingConfig,
        checksum: Option<&str>,
        headers: indexmap::IndexMap<String, String>,
        auth: Auth,
    ) -> DevrcResult<()> {
        let location = Location::Remote {
            url: url.clone(),
            auth: auth.clone(),
        };

        if let Some(cache_ttl) = self.get_cache_ttl() {
            if let Some(content) = crate::cache::load(&url, &loading_config, checksum, &cache_ttl) {
                self.get_logger().debug(
                    &format!("\n==> Loading URL CACHE: `{}` ...", &url),
                    &self.designer.banner(),
                );
                return self.load_from_str(&content, location, Kind::Include, loading_config);
            }
        }

        self.get_logger().debug(
            &format!("\n==> Loading URL: `{}` ...", &url),
            &self.designer.banner(),
        );

        let client = reqwest::blocking::Client::new();
        let mut headers_map: HeaderMap = HeaderMap::new();

        for (key, value) in headers {
            headers_map.insert(
                HeaderName::try_from(key.clone()).map_err(|_| {
                    DevrcError::UrlImportHeadersError {
                        name: key.clone(),
                        value: value.clone(),
                    }
                })?,
                HeaderValue::try_from(value.clone()).map_err(|_| {
                    DevrcError::UrlImportHeadersError {
                        name: key.clone(),
                        value: value.clone(),
                    }
                })?,
            );
        }

        if let Some((key, value)) = auth.get_header() {
            headers_map.insert(
                HeaderName::try_from(key.clone()).map_err(|_| {
                    DevrcError::UrlImportHeadersError {
                        name: key.clone(),
                        value: value.clone(),
                    }
                })?,
                HeaderValue::try_from(value.clone()).map_err(|_| {
                    DevrcError::UrlImportHeadersError {
                        name: key.clone(),
                        value: value.clone(),
                    }
                })?,
            );
        }

        match client.get(url.as_str()).headers(headers_map).send() {
            Ok(response) if response.status() == 200 => {
                let content = response.text().map_err(|_| DevrcError::RuntimeError)?;
                if let Some(control_checksum) = checksum {
                    let content_checksum = digest(content.as_str());

                    if control_checksum != content_checksum {
                        return Err(DevrcError::UrlImportChecksumError {
                            url: url.as_str().to_string(),
                            control_checksum: control_checksum.to_string(),
                            content_checksum,
                        });
                    }
                }

                match self.load_from_str(&content, location, Kind::Include, loading_config) {
                    Ok(_) => {
                        if self.get_cache_ttl().is_some() {
                            crate::cache::save(&url, &content)?;
                        }
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            Ok(response) => {
                loading_config.log_level.debug(
                    &format!(
                        "Loading FILE error: invalid status code `{:}` ...",
                        response.status()
                    ),
                    &loading_config.designer.banner(),
                );
                Err(DevrcError::UrlImportStatusError {
                    url: url.as_str().to_string(),
                    status: response.status(),
                })
            }
            Err(error) => {
                return Err(DevrcError::UrlImportRequestError {
                    url: url.as_str().to_string(),
                    inner: error,
                })
            }
        }
    }

    pub fn load_include(
        &mut self,
        raw_devrcfile: RawDevrcfile,
        source: Location,
        config: LoadingConfig,
    ) -> DevrcResult<()> {
        for include in raw_devrcfile.include {
            match (include, source.clone()) {
                (Include::Empty, _) => {}
                (
                    Include::File(FileInclude {
                        file,
                        path_resolve: _,
                        checksum: _,
                    }),
                    Location::None | Location::StdIn,
                ) => {
                    let path = utils::get_absolute_path(&file.clone(), None)?;
                    self.load_file(
                        path.to_path_buf(),
                        config.clone().child().with_cache_ttl(self.get_cache_ttl()),
                        Kind::Include,
                    )?;
                }

                (
                    Include::File(FileInclude {
                        file,
                        path_resolve: _,
                        checksum: _,
                    }),
                    Location::LocalFile(ref base),
                ) => {
                    let path = utils::get_absolute_path(&file.clone(), Some(&base.clone()))?;
                    self.load_file(
                        path.to_path_buf(),
                        config.clone().child().with_cache_ttl(self.get_cache_ttl()),
                        Kind::Include,
                    )?;
                }
                (
                    Include::File(FileInclude {
                        file,
                        path_resolve,
                        checksum,
                    }),
                    Location::Remote { url, auth },
                ) => {
                    if file.is_absolute() {
                        let loading_config = config
                            .clone()
                            .with_log_level(self.get_logger())
                            .with_cache_ttl(self.get_cache_ttl());
                        self.load_file(file.to_path_buf(), loading_config, Kind::Include)?;
                    } else {
                        match path_resolve {
                            PathResolve::Relative => {
                                let path = file
                                    .clone()
                                    .into_os_string()
                                    .into_string()
                                    .map_err(|_| DevrcError::RuntimeError)?;
                                let include_url = url
                                    .clone()
                                    .join(&path)
                                    .map_err(|_| DevrcError::RuntimeError)?;

                                let loading_config = config
                                    .clone()
                                    .child()
                                    .with_log_level(self.get_logger())
                                    .with_cache_ttl(self.get_cache_ttl());
                                self.load_url(
                                    include_url,
                                    loading_config,
                                    checksum.as_deref(),
                                    IndexMap::new(),
                                    auth.clone(),
                                )?;
                            }
                            PathResolve::Pwd => {
                                let path = utils::get_absolute_path(
                                    &file.clone(),
                                    env::current_dir().ok().as_ref(),
                                )?;
                                self.load_file(
                                    path.to_path_buf(),
                                    config.clone().child(),
                                    Kind::Include,
                                )?;
                            }
                        }
                    }
                }
                (
                    Include::Url(UrlInclude {
                        url,
                        checksum,
                        headers,
                        ignore_errors,
                        auth: raw_auth,
                    }),
                    _,
                ) => {
                    let parsed_url =
                        Url::parse(&url).map_err(|_| DevrcError::InvalidIncludeUrl(url))?;
                    let auth = Auth::try_from(raw_auth)?;

                    if ignore_errors {
                        self.load_url(
                            parsed_url,
                            config.clone().child().with_cache_ttl(self.get_cache_ttl()),
                            Some(&checksum),
                            headers,
                            auth,
                        )
                        .ok();
                    } else {
                        self.load_url(
                            parsed_url,
                            config.clone().child().with_cache_ttl(self.get_cache_ttl()),
                            Some(&checksum),
                            headers,
                            auth,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn add_loaded_file(&mut self, location: Location) -> DevrcResult<()> {
        self.loaded_locations.push(location);
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
    pub fn list_global_vars(&self) -> DevrcResult<()> {
        println!("List global devrc variables:");
        let scope = ((*self.devrc.scope)
            .try_borrow()
            .map_err(|_| DevrcError::RuntimeError)?)
        .clone();
        self.list_vars(&scope)
    }

    /// Show global environment variables and their computed values
    pub fn list_global_env_vars(&self) -> DevrcResult<()> {
        println!("List global devrc environment variables:");
        let scope = ((*self.devrc.scope)
            .try_borrow()
            .map_err(|_| DevrcError::RuntimeError)?)
        .clone();
        self.list_env_vars(&scope)
    }

    pub fn list_vars(&self, scope: &Scope) -> DevrcResult<()> {
        let max_variable_name_width = scope.variables.get_max_key_width();

        for (name, value) in &scope.variables {
            println!(
                "{:width$}{}{:max_variable_name_width$}{} = \"{}\"",
                "",
                self.designer.variable().prefix(),
                name.get_name(),
                self.designer.variable().suffix(),
                value.get_rendered_value(),
                width = 2,
                max_variable_name_width = max_variable_name_width
            );
        }
        Ok(())
    }

    pub fn list_env_vars(&self, scope: &Scope) -> DevrcResult<()> {
        let max_variable_name_width = scope.environment.get_max_key_width();

        for (name, value) in &scope.environment {
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

            if self.log_level == Some(LogLevel::Debug) {
                println!("Task variables:");
                let scope = task.get_scope(
                    &name,
                    Rc::clone(&self.devrc.scope),
                    &TaskArguments::default(),
                )?;
                self.list_vars(&scope)?;
                println!();

                println!("Task environment variables:");
                self.list_env_vars(&scope)?;
                println!("\n");
            }
        }
        Ok(())
    }

    pub fn get_calculated_scope(&self, _scope: &Scope) {}

    pub fn diagnostic(&mut self, params: Vec<String>) {
        println!("Show diagnostic info:");

        self.rest = params;

        println!(
            "Global defined interpreter: `{:}`",
            &self.devrc.config.interpreter
        );

        if let Some(value) = get_global_devrc_file() {
            if value.exists() {
                println!("Global .devrc exists: {:?}", value);
            }
        }

        if let Some(value) = get_directory_devrc_file() {
            if value.exists() {
                println!("Local directory Devrcfile exists: {:?}", value);
            }
        }

        if let Some(value) = get_local_user_defined_devrc_file() {
            if value.exists() {
                println!("Local user defined Devrcfile.local exists: {:?}", value);
            }
        }

        for file in &self.files {
            if let Ok(file) = get_absolute_path(file, None) {
                if file.exists() {
                    println!("Given Devrcfile exists: {:?}", file);
                } else {
                    println!("Given Devrcfile not exists: {:?}", file);
                }
            } else {
                error!("Given Devrcfile with broken path: {:?}", &file);
            }
        }

        for file in &self.registry.files {
            println!("Loaded locations: {:?}", file.location);
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
