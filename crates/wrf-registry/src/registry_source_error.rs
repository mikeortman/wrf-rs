use std::fmt;

use crate::preprocessor::RegistryPreprocessError;
use crate::{RegistryParseError, SourceLocation};

/// Failure loading a Registry source through preprocessing and parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistrySourceError {
    /// Include expansion or conditional filtering failed.
    Preprocess(RegistryPreprocessError),
    /// The expanded source contains an invalid entry.
    Parse(RegistryParseError),
}

impl RegistrySourceError {
    /// Returns the physical location of the failure.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        match self {
            Self::Preprocess(error) => error.location(),
            Self::Parse(error) => error.location(),
        }
    }
}

impl From<RegistryPreprocessError> for RegistrySourceError {
    fn from(error: RegistryPreprocessError) -> Self {
        Self::Preprocess(error)
    }
}

impl From<RegistryParseError> for RegistrySourceError {
    fn from(error: RegistryParseError) -> Self {
        Self::Parse(error)
    }
}

impl fmt::Display for RegistrySourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preprocess(error) => error.fmt(formatter),
            Self::Parse(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RegistrySourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Preprocess(error) => Some(error),
            Self::Parse(error) => Some(error),
        }
    }
}
