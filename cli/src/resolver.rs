use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};
use url::Url;

use crate::auth::Auth;

#[derive(Debug, Clone, Default)]
pub enum Location {
    #[default]
    None,
    StdIn,
    LocalFile(PathBuf),
    Remote {
        url: Url,
        auth: Auth,
    },
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
            Location::Remote { url, .. } => {
                write!(f, "{:}", url.as_str())
            }
        }
    }
}
