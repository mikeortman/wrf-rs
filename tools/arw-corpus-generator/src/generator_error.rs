use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub(crate) enum GeneratorError {
    InvalidArguments { usage: String },
    Io(io::Error),
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArguments { usage } => formatter.write_str(usage),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for GeneratorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidArguments { .. } => None,
            Self::Io(error) => Some(error),
        }
    }
}

impl From<io::Error> for GeneratorError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub(crate) type GeneratorResult<T> = Result<T, GeneratorError>;
