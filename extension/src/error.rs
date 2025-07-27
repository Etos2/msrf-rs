use std::{fmt::Display};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedType(u16),
    InvalidValueLength,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedType(id) => write!(f, "unexpected type {id}"),
            Error::InvalidValueLength => write!(f, "value too small"),
        }
    }
}

impl std::error::Error for Error {}