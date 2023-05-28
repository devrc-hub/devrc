use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ExecKind {
    Empty,
    String(String),
    // Complex(indexmap::IndexMap<String, String>),
    List(Vec<String>),
}

impl ExecKind {}

impl Default for ExecKind {
    fn default() -> Self {
        Self::Empty
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_name() {}
}
