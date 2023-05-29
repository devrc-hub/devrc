#[derive(Debug, Clone, Default)]
pub enum PluginOption {
    #[default]
    None,
    String(String),
    List(Vec<PluginOption>),
    Map(indexmap::IndexMap<String, PluginOption>),
}
