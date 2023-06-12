use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub enum AuthType {
    #[default]
    Empty,
    Bearer,
    BasicAuth,
}

impl From<String> for AuthType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "basic" => AuthType::BasicAuth,
            "bearer" => AuthType::Bearer,
            _ => AuthType::default(),
        }
    }
}

impl<'de> Deserialize<'de> for AuthType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)?.into())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetrcAuth {
    #[serde(rename(deserialize = "machine"))]
    pub host: String,
    pub login: String,

    #[serde(rename(deserialize = "type"))]
    pub auth_type: AuthType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetrcAuthHeader {
    #[serde(rename(deserialize = "machine"))]
    pub host: String,
    pub login: String,
    pub header: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(untagged)]
pub enum Auth {
    #[default]
    Empty,
    NetrcAuth(NetrcAuth),
    NetrcAuthHeader(NetrcAuthHeader),
}
