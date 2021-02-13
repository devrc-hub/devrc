use crate::{errors::DevrcResult, scope::Scope, template::render_string};

pub trait Evaluatable {
    // TODO: create default implementation for Variables and Environment variables
    // Evaluate variable body or environment variable
    fn evaluate(&self, name: &str, parent_scope: &Scope) -> DevrcResult<String>;
}

impl Evaluatable for String {
    fn evaluate(&self, name: &str, scope: &Scope) -> DevrcResult<String> {
        render_string(name, &self, scope)
    }
}

impl Evaluatable for &String {
    fn evaluate(&self, name: &str, scope: &Scope) -> DevrcResult<String> {
        render_string(name, &self, scope)
    }
}
