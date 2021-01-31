
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Examples {
    String(String),
    List(Vec<Examples>)
}
