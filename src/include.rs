use crate::de::deserialize_some;
use serde::Deserialize;
use std::path::PathBuf;

use crate::resolver::PathResolve;

#[derive(Debug, Deserialize, Clone)]
pub struct StringFileInclude(pub String);

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FileInclude {
    pub file: PathBuf,

    #[serde(default)]
    pub path_resolve: PathResolve,

    #[serde(default, deserialize_with = "deserialize_some")]
    pub checksum: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UrlInclude {
    pub url: String,

    pub checksum: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(untagged)]
pub enum Include {
    #[default]
    Empty,
    // Simple(StringFileInclude),
    File(FileInclude),
    Url(UrlInclude),
}
