use serde::{Deserialize};



#[derive(Debug, Deserialize, Clone)]
pub struct ComplexParam {
    pub default: Option<String>,
    pub desc: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ParamValue {
    Required,
    Default(String),
    Complex(ComplexParam),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Params {
    #[serde(flatten)]
    pub params: indexmap::IndexMap<String, ParamValue>,
}

impl Default for Params {
    fn default() -> Self {
        let params = indexmap::IndexMap::new();
        Self { params }
    }
}
