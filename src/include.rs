use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct StringFileInclude(pub String);

#[derive(Debug, Deserialize, Clone)]
pub struct FileInclude {
    pub file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteInclude {
    pub remote: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum IncludeFiles {
    Empty,
    Simple(StringFileInclude),
    File(FileInclude),
    Remote(RemoteInclude),
    List(Vec<IncludeFiles>),
}

#[derive(Debug, Deserialize, Clone)]
pub struct IncludeFilesWrapper(pub IncludeFiles);

impl Default for IncludeFilesWrapper {
    fn default() -> Self {
        Self(IncludeFiles::Empty)
    }
}
