use std::fmt;

use crate::SourceLocation;

/// Specific reason package-selected scalar layouts could not be resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryResolutionErrorKind {
    /// The caller supplied the same runtime-configuration value twice.
    DuplicateRuntimeConfigurationChoice {
        /// Repeated runtime-configuration symbol.
        name: String,
    },
    /// A supplied choice does not name a parsed runtime configuration.
    UnknownRuntimeConfigurationChoice {
        /// Undefined runtime-configuration symbol.
        name: String,
    },
    /// A package condition refers to an undefined runtime configuration.
    UnknownPackageConditionConfiguration {
        /// Undefined runtime-configuration symbol.
        name: String,
    },
    /// A package condition refers to a configuration that is not integer-valued.
    NonIntegerPackageConditionConfiguration {
        /// Non-integer runtime-configuration symbol.
        name: String,
    },
    /// A runtime-configuration symbol is declared more than once.
    DuplicateRuntimeConfiguration {
        /// Repeated runtime-configuration symbol.
        name: String,
    },
    /// A four-dimensional scalar member has no owning array association.
    MissingScalarArrayAssociation {
        /// State member lacking an association.
        member_name: String,
    },
    /// A scalar-array definition contains the same member more than once.
    DuplicateScalarArrayDefinitionMember {
        /// Owning scalar array.
        scalar_array_name: String,
        /// Repeated state/member name.
        member_name: String,
    },
    /// A scalar-array definition does not begin with WRF's reserved `-` member.
    MissingReservedScalarArrayMember {
        /// Invalid scalar-array definition.
        scalar_array_name: String,
    },
    /// A package attempts to activate the reserved `-` placeholder as data.
    ReservedScalarArrayMemberActivation {
        /// Owning scalar-array name.
        scalar_array_name: String,
    },
    /// A selected package names no parsed four-dimensional scalar array.
    UnknownScalarArray {
        /// Undefined scalar-array name.
        scalar_array_name: String,
    },
    /// A selected package names no member of the associated scalar array.
    UnknownScalarArrayMember {
        /// Owning scalar-array name.
        scalar_array_name: String,
        /// Undefined member name.
        member_name: String,
    },
}

/// Typed cross-entry Registry resolution failure with physical source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryResolutionError {
    location: SourceLocation,
    kind: RegistryResolutionErrorKind,
}

/// Result returned by package-selected scalar-layout resolution.
pub type RegistryResolutionResult<T> = Result<T, RegistryResolutionError>;

impl RegistryResolutionError {
    pub(crate) const fn new(location: SourceLocation, kind: RegistryResolutionErrorKind) -> Self {
        Self { location, kind }
    }

    /// Returns the physical source position responsible for the failure.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the typed cross-entry failure reason.
    #[must_use]
    pub const fn kind(&self) -> &RegistryResolutionErrorKind {
        &self.kind
    }
}

impl fmt::Display for RegistryResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.location)?;
        match &self.kind {
            RegistryResolutionErrorKind::DuplicateRuntimeConfigurationChoice { name } => {
                write!(
                    formatter,
                    "runtime configuration `{name}` was supplied more than once"
                )
            }
            RegistryResolutionErrorKind::UnknownRuntimeConfigurationChoice { name } => {
                write!(
                    formatter,
                    "runtime configuration choice `{name}` is not declared"
                )
            }
            RegistryResolutionErrorKind::UnknownPackageConditionConfiguration { name } => {
                write!(
                    formatter,
                    "package condition refers to undefined configuration `{name}`"
                )
            }
            RegistryResolutionErrorKind::NonIntegerPackageConditionConfiguration { name } => {
                write!(
                    formatter,
                    "package condition configuration `{name}` is not integer-valued"
                )
            }
            RegistryResolutionErrorKind::DuplicateRuntimeConfiguration { name } => {
                write!(
                    formatter,
                    "runtime configuration `{name}` is declared more than once"
                )
            }
            RegistryResolutionErrorKind::MissingScalarArrayAssociation { member_name } => {
                write!(
                    formatter,
                    "scalar member `{member_name}` has no owning array association"
                )
            }
            RegistryResolutionErrorKind::DuplicateScalarArrayDefinitionMember {
                scalar_array_name,
                member_name,
            } => write!(
                formatter,
                "scalar array `{scalar_array_name}` defines member `{member_name}` more than once"
            ),
            RegistryResolutionErrorKind::MissingReservedScalarArrayMember { scalar_array_name } => {
                write!(
                    formatter,
                    "scalar array `{scalar_array_name}` does not begin with the reserved `-` member"
                )
            }
            RegistryResolutionErrorKind::ReservedScalarArrayMemberActivation {
                scalar_array_name,
            } => write!(
                formatter,
                "package attempts to activate reserved member `-` of scalar array `{scalar_array_name}`"
            ),
            RegistryResolutionErrorKind::UnknownScalarArray { scalar_array_name } => {
                write!(
                    formatter,
                    "package refers to undefined scalar array `{scalar_array_name}`"
                )
            }
            RegistryResolutionErrorKind::UnknownScalarArrayMember {
                scalar_array_name,
                member_name,
            } => write!(
                formatter,
                "package refers to undefined member `{member_name}` of scalar array `{scalar_array_name}`"
            ),
        }
    }
}

impl std::error::Error for RegistryResolutionError {}
