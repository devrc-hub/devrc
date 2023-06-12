use crate::{de::deserialize_some, raw::auth::Auth};
use serde::Deserialize;
use std::path::PathBuf;

use crate::resolver::PathResolve;

pub(crate) fn get_default_skip_on_error() -> bool {
    false
}

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

    #[serde(default)]
    pub headers: indexmap::IndexMap<String, String>,

    #[serde(default = "get_default_skip_on_error")]
    pub ignore_errors: bool,

    #[serde(default)]
    pub auth: Auth,
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
