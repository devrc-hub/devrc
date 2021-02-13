use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Examples {
    String(String),
    List(Vec<Examples>),
}
