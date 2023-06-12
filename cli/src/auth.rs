use std::{convert::TryFrom, fs};

use netrc_rs::{Machine, Netrc};

use crate::{
    errors::DevrcError,
    netrc::get_user_defined_netrc_path,
    raw::auth::{NetrcAuth, NetrcAuthHeader},
};

pub const HEADER_AUTHORIZATION: &str = "Authorization";

#[derive(Debug, Clone, Default)]
pub enum AuthType {
    #[default]
    None,
    Bearer,
    BasicAuth,
    Header(String),
}

#[derive(Debug, Clone, Default)]
pub enum Auth {
    #[default]
    Empty,
    Loaded {
        machine: Machine,
        auth_type: AuthType,
    },
}

impl TryFrom<crate::raw::auth::Auth> for Auth {
    type Error = DevrcError;

    fn try_from(value: crate::raw::auth::Auth) -> Result<Self, Self::Error> {
        let file = get_user_defined_netrc_path().ok_or(DevrcError::NetrcNotFound)?;
        let content = fs::read_to_string(file).map_err(DevrcError::IoError)?;
        let netrc = Netrc::parse(content, false).map_err(DevrcError::NetrcParsingError)?;

        match value {
            crate::raw::auth::Auth::Empty => Ok(Auth::Empty),
            crate::raw::auth::Auth::NetrcAuth(NetrcAuth {
                host,
                login,
                auth_type,
            }) => match get_machine(&netrc, &host, &login) {
                Some(machine) => Ok(Auth::Loaded {
                    machine,
                    auth_type: auth_type.into(),
                }),
                None => Ok(Auth::Empty),
            },
            crate::raw::auth::Auth::NetrcAuthHeader(NetrcAuthHeader {
                host,
                login,
                header,
            }) => match get_machine(&netrc, &host, &login) {
                Some(machine) => Ok(Auth::Loaded {
                    machine,
                    auth_type: AuthType::Header(header),
                }),
                None => Ok(Auth::Empty),
            },
        }
    }
}

impl From<crate::raw::auth::AuthType> for AuthType {
    fn from(value: crate::raw::auth::AuthType) -> Self {
        match value {
            crate::raw::auth::AuthType::Empty => AuthType::None,
            crate::raw::auth::AuthType::Bearer => AuthType::Bearer,
            crate::raw::auth::AuthType::BasicAuth => AuthType::BasicAuth,
        }
    }
}

fn get_machine(netrc: &Netrc, host: &str, login: &str) -> Option<Machine> {
    let mut default: Option<Machine> = None;

    for machine in &netrc.machines {
        match (machine.name.as_ref(), machine.login.as_ref()) {
            (Some(record_host), Some(record_login))
                if record_host.as_str() == host && record_login.as_str() == login =>
            {
                return Some(machine.clone())
            }
            (None, Some(record_login)) if record_login.as_str() == login => {
                default = Some(machine.clone())
            }
            (_, _) => {}
        }
    }

    default
}

impl Auth {
    pub fn get_header(&self) -> Option<(String, String)> {
        if let Auth::Loaded { machine, auth_type } = self {
            let password = match &machine.password {
                Some(password) => password,
                None => return None,
            };
            let login = match &machine.login {
                Some(login) => login,
                None => return None,
            };

            return match auth_type {
                AuthType::Bearer => Some((
                    HEADER_AUTHORIZATION.to_string(),
                    format!("Bearer {}", password),
                )),
                AuthType::BasicAuth => {
                    let creds = format!("{:}:{:}", login, password);
                    let b64 =
                        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, creds);
                    Some((HEADER_AUTHORIZATION.to_string(), format!("Basic {}", b64)))
                }
                AuthType::Header(header) => Some((header.to_owned(), password.to_owned())),
                AuthType::None => None,
            };
        }
        None
    }
}
