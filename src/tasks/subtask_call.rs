
use serde::Deserialize;

use crate::{environment::RawEnvironment, variables::RawVariables};



#[derive(Debug, Deserialize, Clone, Default)]
pub struct SubtaskCall {
    pub name: String,

    #[serde(default)]
    pub environment: RawEnvironment<String>,

    #[serde(default)]
    pub variables: RawVariables,

}
