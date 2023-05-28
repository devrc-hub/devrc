use std::any::Any;

use crate::config::Config;

/// A plugin which allows you to add extra functionality to the REST client.
pub trait Plugin: Any + Send + Sync {
    /// Get a name describing the `Plugin`.
    fn name(&self) -> &'static str;
    /// A callback fired immediately after the plugin is loaded. Usually used
    /// for initialization.
    fn on_plugin_load(&mut self, _config: Config) {}
    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn on_plugin_unload(&self) {}
}
