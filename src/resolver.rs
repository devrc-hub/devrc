use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};
use url::Url;

#[derive(Debug, Clone, Default)]
pub enum Location {
    #[default]
    None,
    StdIn,
    LocalFile(PathBuf),
    Url(Url),
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum PathResolve {
    #[default]
    #[serde(rename(deserialize = "relative"))]
    Relative,

    #[serde(rename(deserialize = "pwd"))]
    Pwd,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::None => Ok(()),
            Location::StdIn => Ok(()),
            Location::LocalFile(value) => {
                write!(f, "{:}", value.display())
            }
            Location::Url(value) => {
                write!(f, "{:}", value.as_str())
            }
        }
    }
}
