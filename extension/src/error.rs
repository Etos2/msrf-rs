use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesError {
    UnexpectedType(u16),
    UnexpectedLength(u64),
}

impl Display for DesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedType(id) => write!(f, "unexpected type id ({id:#06x})"),
            Self::UnexpectedLength(len) => write!(f, "record too small ({len})"),
        }
    }
}

impl std::error::Error for DesError {}
