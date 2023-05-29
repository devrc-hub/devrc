use std::ffi::OsStr;

use devrc_core::{logging::LogLevel, workshop::Designer};
use libloading::{Library, Symbol};

use crate::{
    config::{Config, ExecutionConfig},
    errors::{DevrcPluginError, DevrcPluginResult},
    plugin::Plugin,
};

pub trait ExecutionPlugin: Plugin {
    fn execute(
        &self,
        execution_config: ExecutionConfig,
        code: &str,
        environment: &indexmap::IndexMap<String, String>,
    ) -> DevrcPluginResult<i32>;
}

/// Declare a plugin type and its constructor.
///
/// # Notes
///
/// This works by automatically generating an `extern "C"` function with a
/// pre-defined signature and symbol name. Therefore you will only be able to
/// declare one plugin per library.
#[macro_export]
macro_rules! declare_execution_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::ExecutionPlugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::ExecutionPlugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

#[derive(Default)]
pub struct ExecutionPluginManager {
    plugins: Vec<(String, Box<dyn ExecutionPlugin>)>,
    loaded_libraries: Vec<Library>,
    designer: Designer,
    logger: LogLevel,
}

impl std::fmt::Debug for ExecutionPluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager").finish()
    }
}

impl ExecutionPluginManager {
    pub fn new() -> ExecutionPluginManager {
        ExecutionPluginManager {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
            designer: Designer::default(),
            logger: LogLevel::default(),
        }
    }

    pub fn setup_logger(&mut self, logger: LogLevel) {
        self.logger = logger;
    }

    /// # Safety
    ///
    /// This function load plugin from dynamic library
    pub unsafe fn load_plugin<P: AsRef<OsStr>>(
        &mut self,
        name: &str,
        filename: P,
        logger: LogLevel,
    ) -> DevrcPluginResult<()> {
        type PluginCreate = unsafe fn() -> *mut dyn ExecutionPlugin;

        let lib = Library::new(filename.as_ref())?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")?;
        let boxed_raw = constructor();

        let mut plugin = Box::from_raw(boxed_raw);

        logger.debug(
            &format!(
                "\n==> Loading PLUGIN: `{}` as `{}` from `{:?}` ...",
                plugin.name(),
                name,
                filename.as_ref()
            ),
            &self.designer.banner(),
        );

        plugin.on_plugin_load(Config {
            logger,
            designer: self.designer,
        });
        self.plugins.push((name.to_string(), plugin));
        Ok(())
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        self.logger
            .debug("\n==> Unloading PLUGINS ...", &self.designer.banner());

        for (name, plugin) in self.plugins.drain(..) {
            self.logger.debug(
                &format!("\n==> Upload PLUGIN: `{}` named `{}` ", plugin.name(), name),
                &self.designer.banner(),
            );
            plugin.on_plugin_unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }

    pub fn get_plugin(
        &mut self,
        plugin_name: &str,
    ) -> DevrcPluginResult<&Box<dyn ExecutionPlugin>> {
        for (name, plugin) in &self.plugins {
            if name == plugin_name {
                return Ok(plugin);
            }
        }

        Err(DevrcPluginError::NotFound(plugin_name.to_string()))
    }
}

impl Drop for ExecutionPluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}
