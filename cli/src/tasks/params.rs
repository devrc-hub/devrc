use std::{cmp::Ordering, convert::TryFrom};

use serde::Deserialize;

use crate::{
    errors::{DevrcError, DevrcResult},
    workshop::Designer,
};

use super::params_parser::parse_params_string;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ComplexParam {
    pub default: Option<String>,
    pub desc: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ParamValue {
    Required,
    Default(String),
    // Complex(ComplexParam),
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Params {
    #[serde(flatten)]
    pub params: indexmap::IndexMap<String, ParamValue>,
}

impl Params {
    pub fn add(&mut self, key: &str, value: &ParamValue) -> DevrcResult<()> {
        self.params.insert(key.to_string(), value.to_owned());

        Ok(())
    }

    pub fn update(&mut self, params: Params) -> DevrcResult<()> {
        for (key, value) in params.params.iter() {
            self.add(key, value)?;
        }
        Ok(())
    }

    pub fn sort(&mut self) -> DevrcResult<()> {
        self.params.sort_by(|_, v1, _, v2| match (v1, v2) {
            (ParamValue::Required, ParamValue::Required) => Ordering::Equal,
            (ParamValue::Required, ParamValue::Default(_)) => Ordering::Less,
            (ParamValue::Default(_), ParamValue::Required) => Ordering::Greater,
            (ParamValue::Default(_), ParamValue::Default(_)) => Ordering::Equal,
        });

        // self.params = sorted;

        Ok(())
    }

    pub fn merge(&mut self, mut params: Params) -> DevrcResult<()> {
        // Parametes from `params` have priority over
        // parameters from task name

        for (key, value) in self.params.iter() {
            if params.params.contains_key(key) {
                // There are parameters name which are cpecified
                // in task name and params key
                return Err(DevrcError::OverlappingParameters);
            }
            params.add(key, value)?;
        }

        self.params = params.params;

        self.sort()?;

        Ok(())
    }

    pub fn format_help_string(&self, designer: &Designer) -> DevrcResult<String> {
        let mut parts = Vec::new();

        for (key, value) in self.params.iter() {
            match value {
                ParamValue::Required => parts.push(format!(
                    "{}{}{}",
                    designer.parameter_name().prefix(),
                    key,
                    designer.parameter_name().suffix()
                )),
                ParamValue::Default(default) => {
                    let help = format!(
                        "{}{}{}=\"{}{}{}\"",
                        designer.parameter_name().prefix(),
                        key,
                        designer.parameter_name().suffix(),
                        designer.parameter_value().prefix(),
                        default,
                        designer.parameter_value().suffix()
                    );

                    parts.push(help)
                }
            }
        }

        Ok(parts.join(" "))
    }

    // pub fn evaluate(&self, parent_scope: &Scope) -> DevrcResult<indexmap::IndexMap<String, String>>{
    //     let mut local_scope = parent_scope.clone();
    //     let mut vars = indexmap::IndexMap::new();
    //     for (key, value) in &self.params {
    //         match value.evaluate(&key, &local_scope) {
    //             Ok(value) => {
    //                 local_scope.insert_var(&key, &value);
    //                 vars.insert(key.clone(), value)
    //             }
    //             Err(error) => return Err(error),
    //         };
    //     }
    //     Ok(vars)
    // }
}

impl TryFrom<String> for Params {
    type Error = DevrcError;

    fn try_from(value: String) -> DevrcResult<Self> {
        parse_params_string(&value)
    }
}

impl Default for Params {
    fn default() -> Self {
        let params = indexmap::IndexMap::new();
        Self { params }
    }
}
