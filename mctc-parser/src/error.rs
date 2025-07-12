use std::{
    error::Error,
    fmt::{self, Display},
};

pub type CodecResult<T> = Result<T, CodecError>;

#[derive(Debug)]
pub enum CodecError {
    Needed(usize),
    Err(Box<dyn Error>),
}

impl Error for CodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CodecError::Needed(_) => None,
            CodecError::Err(error) => Some(error.as_ref()),
        }
    }
}

impl Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::Needed(bytes) => write!(f, "need {bytes} bytes"),
            CodecError::Err(e) => write!(f, "{e}"),
        }
    }
}

impl CodecError {
    pub fn from_custom<E: Error + 'static>(error: E) -> CodecError {
        CodecError::Err(Box::new(error))
    }
}