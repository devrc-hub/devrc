
use std::{fmt, io::{self, Error as IoError}};
use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
use serde_yaml::{Error as SerdeYamlError};

use tera::{ErrorKind as TeraErrorKing, Error as TeraError};

pub type DevrcResult<T> = Result<T, DevrcError>;


#[derive(Debug)]
pub enum DevrcError {
    Dotenv,
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
    Code {
        code: i32
    },
    RuntimeError
}


impl Display for DevrcError {

    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {

        write!(f, "{}: \n", "devrc error");

        match self {
            // TODO: add source context to error
            DevrcError::RenderError(terra_error) => {
                match TeraError::source(&terra_error) {
                    Some(value) => {
                        write!(f, "{:}", &value);
                    },
                    value => {
                        println!("another value");
                    }
                }
                // write!(f, "{}: ", terra_error);
            },
            DevrcError::Code {code} => {
                write!(f, "Recipe failed with code {:}", code);
            },
            _ => {

            }
        }
        Ok(())
    }

}


impl From<dotenv::Error> for DevrcError {

    fn from(error: dotenv::Error) -> DevrcError {
        DevrcError::Dotenv
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


impl StdError for DevrcError {

}
