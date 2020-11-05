use crate::error::MavenError::InvalidInputError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type Result<T> = std::result::Result<T, MavenError>;

#[derive(Eq, PartialEq)]
pub enum MavenError {
    InvalidInputError { message: String },
}

impl Debug for MavenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MavenError::InvalidInputError { message } => write!(f, "Maven Error: {}", message),
        }
    }
}

impl Display for MavenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl Error for MavenError {}

impl MavenError {
    pub fn invalid_input<T, S: Into<String>>(message: S) -> Result<T> {
        Err(InvalidInputError {
            message: message.into(),
        })
    }
}
