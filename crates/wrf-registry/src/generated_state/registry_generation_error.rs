use std::fmt;

/// Failure to generate the selected WRF Registry artifacts safely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryGenerationError {
    /// One of WRF's three spatial model orders has no dimension.
    MissingStandardDimensionOrder {
        /// Missing one-based order.
        order: u8,
    },
    /// More than one standard-domain dimension claims the same model order.
    DuplicateStandardDimensionOrder {
        /// Duplicated one-based order.
        order: u8,
    },
    /// Four-dimensional scalar-array members are parsed but not yet generated.
    UnsupportedScalarArrayMember {
        /// State symbol requiring the unsupported generator path.
        state_name: String,
    },
}

/// Result returned by Registry artifact generation.
pub type RegistryGenerationResult<T> = Result<T, RegistryGenerationError>;

impl fmt::Display for RegistryGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStandardDimensionOrder { order } => {
                write!(formatter, "no standard-domain dimension has order {order}")
            }
            Self::DuplicateStandardDimensionOrder { order } => {
                write!(
                    formatter,
                    "multiple standard-domain dimensions have order {order}"
                )
            }
            Self::UnsupportedScalarArrayMember { state_name } => write!(
                formatter,
                "state `{state_name}` uses a four-dimensional scalar-array form outside this generator slice"
            ),
        }
    }
}

impl std::error::Error for RegistryGenerationError {}
