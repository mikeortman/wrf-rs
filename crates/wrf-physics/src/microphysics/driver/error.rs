use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{
    KesslerMicrophysicsAxis, KesslerMicrophysicsError, MicrophysicsScheme, MoistureSpecies,
};

/// Failure produced while configuring or dispatching the microphysics driver.
#[derive(Clone, Debug, PartialEq)]
pub enum MicrophysicsDriverError {
    /// The `mp_physics` namelist value names a scheme that is not ported.
    UnsupportedScheme {
        /// Rejected `mp_physics` value.
        mp_physics: i32,
    },
    /// A moisture package must carry at least one species.
    EmptyMoisturePackage,
    /// A moisture package names the same species twice.
    DuplicateMoistureSpecies {
        /// Species listed more than once.
        species: MoistureSpecies,
    },
    /// The selected scheme requires a species the package does not carry.
    MissingMoistureSpecies {
        /// Species the scheme requires.
        species: MoistureSpecies,
    },
    /// The moisture field slice does not match the package species count.
    MoistureFieldCountMismatch {
        /// Species carried by the package.
        expected: usize,
        /// Fields supplied by the caller.
        actual: usize,
    },
    /// One moisture species field has the wrong allocation shape.
    MoistureFieldShapeMismatch {
        /// Species whose field shape differs.
        species: MoistureSpecies,
        /// Shape required by the driver domain.
        expected: GridShape,
        /// Actual field shape.
        actual: GridShape,
    },
    /// A domain range contains no points.
    EmptyDomainRange {
        /// Axis containing the empty range.
        axis: KesslerMicrophysicsAxis,
    },
    /// A domain range extends beyond its allocated field dimension.
    DomainRangeOutOfBounds {
        /// Axis containing the invalid range.
        axis: KesslerMicrophysicsAxis,
        /// Exclusive requested range end.
        range_end: usize,
        /// Allocated number of points on the axis.
        field_extent: usize,
    },
    /// Reusable scratch was created for a different microphysics scheme.
    WorkspaceSchemeMismatch {
        /// Scheme selected by the driver.
        driver_scheme: MicrophysicsScheme,
        /// Scheme for which the workspace was created.
        workspace_scheme: MicrophysicsScheme,
    },
    /// The dispatched scheme kernel rejected its configuration or execution.
    Kernel(KesslerMicrophysicsError),
}

/// Result alias for microphysics driver configuration and dispatch.
pub type MicrophysicsDriverResult<T> = Result<T, MicrophysicsDriverError>;

impl fmt::Display for MicrophysicsDriverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedScheme { mp_physics } => write!(
                formatter,
                "mp_physics {mp_physics} selects a microphysics scheme that is not ported"
            ),
            Self::EmptyMoisturePackage => {
                formatter.write_str("moisture species package carries no species")
            }
            Self::DuplicateMoistureSpecies { species } => write!(
                formatter,
                "moisture species package names {species} more than once"
            ),
            Self::MissingMoistureSpecies { species } => write!(
                formatter,
                "microphysics scheme requires {species}, which the package does not carry"
            ),
            Self::MoistureFieldCountMismatch { expected, actual } => write!(
                formatter,
                "moisture package carries {expected} species but {actual} fields were supplied"
            ),
            Self::MoistureFieldShapeMismatch {
                species,
                expected,
                actual,
            } => write!(
                formatter,
                "{species} field shape {actual:?} does not match expected shape {expected:?}"
            ),
            Self::EmptyDomainRange { axis } => {
                write!(formatter, "microphysics {axis} domain range is empty")
            }
            Self::DomainRangeOutOfBounds {
                axis,
                range_end,
                field_extent,
            } => write!(
                formatter,
                "microphysics {axis} domain range ends at {range_end}, beyond field extent {field_extent}"
            ),
            Self::WorkspaceSchemeMismatch {
                driver_scheme,
                workspace_scheme,
            } => write!(
                formatter,
                "microphysics {workspace_scheme:?} workspace cannot execute the {driver_scheme:?} driver"
            ),
            Self::Kernel(error) => write!(formatter, "microphysics kernel failed: {error}"),
        }
    }
}

impl Error for MicrophysicsDriverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            _ => None,
        }
    }
}

impl From<KesslerMicrophysicsError> for MicrophysicsDriverError {
    fn from(error: KesslerMicrophysicsError) -> Self {
        Self::Kernel(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_unsupported_scheme_value() {
        let error = MicrophysicsDriverError::UnsupportedScheme { mp_physics: 8 };

        assert_eq!(
            error.to_string(),
            "mp_physics 8 selects a microphysics scheme that is not ported"
        );
    }

    #[test]
    fn kernel_errors_convert_and_expose_their_source() {
        let error = MicrophysicsDriverError::from(KesslerMicrophysicsError::WorkerIndexUnavailable);

        assert_eq!(
            error,
            MicrophysicsDriverError::Kernel(KesslerMicrophysicsError::WorkerIndexUnavailable)
        );
        assert!(error.source().is_some());
    }
}
