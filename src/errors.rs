use serde_yaml::Error as SerdeYamlError;
use std::{
    error::Error as StdError,
    fmt,
    fmt::{Display, Formatter},
    io::Error as IoError,
};

use tera::Error as TeraError;

use dotenv::{self, Error as DotenvError};

pub type DevrcResult<T> = Result<T, DevrcError>;

#[derive(Debug)]
pub enum DevrcError {
    Dotenv(DotenvError),
    NotExists,
    GlobalNotExists,
    LocalNotExists,
    RenderError(TeraError),
    EmptyVariable,
    EmptyEnvironmentVariable,
    IoError(IoError),
    YamlParseError(SerdeYamlError),
    TaskNotFound,
    NotImplemented,
    Signal,
    Code { code: i32 },
    RuntimeError,
}

impl Display for DevrcError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "devrc error: \n")?;

        match self {
            // TODO: add source context to error
            DevrcError::RenderError(terra_error) => {
                match TeraError::source(&terra_error) {
                    Some(value) => {
                        write!(f, "{:}", &value)?;
                    }
                    _value => {
                        println!("another value");
                    }
                }
                // write!(f, "{}: ", terra_error);
            }
            DevrcError::Code { code } => {
                write!(f, "Recipe failed with code {:}", code)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl From<DotenvError> for DevrcError {
    fn from(error: DotenvError) -> DevrcError {
        DevrcError::Dotenv(error)
    }
}

impl From<tera::Error> for DevrcError {
    fn from(error: tera::Error) -> DevrcError {
        DevrcError::RenderError(error)
    }
}

impl From<IoError> for DevrcError {
    fn from(error: IoError) -> DevrcError {
        DevrcError::IoError(error)
    }
}

impl StdError for DevrcError {}
