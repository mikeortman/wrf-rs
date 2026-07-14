use std::fmt;

use crate::SourceLocation;

/// Specific reason a Registry source could not be represented safely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryParseErrorKind {
    /// A final physical line requests continuation but no following line exists.
    DanglingContinuation,
    /// A logical line contains an opening quote without a matching close quote.
    UnbalancedQuote,
    /// The entry category is outside the documented first parser slice.
    UnsupportedEntry {
        /// Unsupported first token.
        entry_kind: String,
    },
    /// An entry does not provide its exact positional field count.
    UnexpectedTokenCount {
        /// Entry category whose shape was invalid.
        entry_kind: String,
        /// Required number of positional tokens.
        expected: usize,
        /// Number of tokens found after WRF-compatible tokenization.
        actual: usize,
    },
    /// A package does not provide the supported three-to-five positional fields.
    UnexpectedPackageTokenCount {
        /// Number of tokens found after WRF-compatible tokenization.
        actual: usize,
    },
    /// A package name is empty or uses the reserved absent-field marker.
    InvalidPackageName,
    /// A package symbol is defined more than once.
    DuplicatePackage {
        /// Repeated package symbol.
        name: String,
    },
    /// A package condition is not a single `name==signed_integer` expression.
    InvalidPackageCondition {
        /// Invalid source token.
        value: String,
    },
    /// The signed integer on the right side of a package condition is invalid.
    InvalidPackageChoice {
        /// Invalid source token.
        value: String,
    },
    /// The legacy package state-variable field is outside this slice.
    UnsupportedPackageStateVariables {
        /// Unsupported source token.
        value: String,
    },
    /// A semicolon-delimited package variable group is empty or malformed.
    InvalidPackageVariableGroup {
        /// Invalid group text.
        value: String,
    },
    /// A package variable group names no scalar array before `:`.
    EmptyPackageVariableGroupName,
    /// A package variable group contains an empty comma-delimited member.
    EmptyPackageVariableGroupMember {
        /// Owning scalar-array name.
        scalar_array_name: String,
    },
    /// A package activates the same scalar-array member more than once.
    DuplicatePackageVariableGroupMember {
        /// Owning scalar-array name.
        scalar_array_name: String,
        /// Repeated member name.
        member_name: String,
    },
    /// A dimension symbol is defined more than once.
    DuplicateDimension {
        /// Repeated dimension symbol.
        name: String,
    },
    /// A state refers to a dimension not defined earlier in source order.
    UnknownDimension {
        /// Undefined dimension symbol.
        name: String,
    },
    /// A dimension order is not a valid supported integer/order.
    InvalidDimensionOrder {
        /// Invalid source token.
        value: String,
    },
    /// A `standard_domain`, `constant=`, or `namelist=` expression is invalid.
    InvalidDimensionLength {
        /// Invalid source token.
        value: String,
    },
    /// A dimension axis is not `x`, `y`, `z`, `c`, or `-`.
    InvalidCoordinateAxis {
        /// Invalid source token.
        value: String,
    },
    /// A field uses a type outside WRF's built-in Registry types.
    InvalidValueType {
        /// Invalid source token.
        value: String,
    },
    /// A state time-level count is not a positive `u8`.
    InvalidTimeLevels {
        /// Invalid source token.
        value: String,
    },
    /// A dimension string uses malformed or not-yet-supported modifiers.
    UnsupportedStateDimensionSyntax {
        /// Unsupported source token.
        value: String,
    },
    /// A state staggering string contains an unknown flag.
    InvalidStaggering {
        /// Invalid source token.
        value: String,
    },
}

/// Typed parse failure with its original physical source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryParseError {
    location: SourceLocation,
    kind: RegistryParseErrorKind,
}

/// Result returned by Registry parsing operations.
pub type RegistryResult<T> = Result<T, RegistryParseError>;

impl RegistryParseError {
    pub(crate) const fn new(location: SourceLocation, kind: RegistryParseErrorKind) -> Self {
        Self { location, kind }
    }

    /// Returns the physical source position attached to the failure.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the typed reason parsing failed.
    #[must_use]
    pub const fn kind(&self) -> &RegistryParseErrorKind {
        &self.kind
    }
}

impl fmt::Display for RegistryParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.location)?;
        match &self.kind {
            RegistryParseErrorKind::DanglingContinuation => {
                formatter.write_str("backslash continuation reaches end of source")
            }
            RegistryParseErrorKind::UnbalancedQuote => {
                formatter.write_str("unbalanced double quote")
            }
            RegistryParseErrorKind::UnsupportedEntry { entry_kind } => {
                write!(formatter, "unsupported Registry entry `{entry_kind}`")
            }
            RegistryParseErrorKind::UnexpectedTokenCount {
                entry_kind,
                expected,
                actual,
            } => write!(
                formatter,
                "`{entry_kind}` expects {expected} tokens but received {actual}"
            ),
            RegistryParseErrorKind::UnexpectedPackageTokenCount { actual } => write!(
                formatter,
                "`package` expects between 3 and 5 tokens but received {actual}"
            ),
            RegistryParseErrorKind::InvalidPackageName => {
                formatter.write_str("package name must not be empty or `-`")
            }
            RegistryParseErrorKind::DuplicatePackage { name } => {
                write!(formatter, "package `{name}` is already defined")
            }
            RegistryParseErrorKind::InvalidPackageCondition { value } => write!(
                formatter,
                "invalid package condition `{value}`; expected `name==signed_integer`"
            ),
            RegistryParseErrorKind::InvalidPackageChoice { value } => {
                write!(formatter, "invalid signed package choice `{value}`")
            }
            RegistryParseErrorKind::UnsupportedPackageStateVariables { value } => write!(
                formatter,
                "package state-variable field `{value}` is not supported by this slice"
            ),
            RegistryParseErrorKind::InvalidPackageVariableGroup { value } => {
                write!(formatter, "invalid package variable group `{value}`")
            }
            RegistryParseErrorKind::EmptyPackageVariableGroupName => {
                formatter.write_str("package variable group name must not be empty")
            }
            RegistryParseErrorKind::EmptyPackageVariableGroupMember { scalar_array_name } => {
                write!(
                    formatter,
                    "package variable group `{scalar_array_name}` contains an empty member"
                )
            }
            RegistryParseErrorKind::DuplicatePackageVariableGroupMember {
                scalar_array_name,
                member_name,
            } => write!(
                formatter,
                "package variable group `{scalar_array_name}` repeats member `{member_name}`"
            ),
            RegistryParseErrorKind::DuplicateDimension { name } => {
                write!(formatter, "dimension `{name}` is already defined")
            }
            RegistryParseErrorKind::UnknownDimension { name } => {
                write!(formatter, "state references undefined dimension `{name}`")
            }
            RegistryParseErrorKind::InvalidDimensionOrder { value } => {
                write!(formatter, "invalid dimension order `{value}`")
            }
            RegistryParseErrorKind::InvalidDimensionLength { value } => {
                write!(formatter, "invalid dimension length `{value}`")
            }
            RegistryParseErrorKind::InvalidCoordinateAxis { value } => {
                write!(formatter, "invalid coordinate axis `{value}`")
            }
            RegistryParseErrorKind::InvalidValueType { value } => {
                write!(formatter, "unsupported Registry value type `{value}`")
            }
            RegistryParseErrorKind::InvalidTimeLevels { value } => {
                write!(formatter, "invalid state time-level count `{value}`")
            }
            RegistryParseErrorKind::UnsupportedStateDimensionSyntax { value } => {
                write!(formatter, "unsupported state dimension string `{value}`")
            }
            RegistryParseErrorKind::InvalidStaggering { value } => {
                write!(formatter, "invalid state staggering flags `{value}`")
            }
        }
    }
}

impl std::error::Error for RegistryParseError {}
