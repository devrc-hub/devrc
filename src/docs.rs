use std::{cmp, collections::HashMap};

use unicode_width::UnicodeWidthStr;

use crate::variables::{VariableKey, VariableValue};

pub trait DocHelper {
    fn get_max_key_width(&self) -> usize;
}

impl DocHelper for indexmap::IndexMap<String, String> {
    fn get_max_key_width(&self) -> usize {
        let mut name_width = 0;
        for (name, _) in self {
            name_width = cmp::max(
                name_width,
                UnicodeWidthStr::width(name.to_string().as_str()),
            );
        }
        name_width
    }
}

impl DocHelper for HashMap<String, String> {
    fn get_max_key_width(&self) -> usize {
        let mut name_width = 0;
        for name in self.keys() {
            name_width = cmp::max(
                name_width,
                UnicodeWidthStr::width(name.to_string().as_str()),
            );
        }
        name_width
    }
}

impl DocHelper for indexmap::IndexMap<VariableKey, VariableValue> {
    fn get_max_key_width(&self) -> usize {
        let mut name_width = 0;
        for (name, _) in self {
            name_width = cmp::max(name_width, UnicodeWidthStr::width(name.get_name().as_str()));
        }
        name_width
    }
}
