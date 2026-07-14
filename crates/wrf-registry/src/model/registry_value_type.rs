use std::fmt;

/// Built-in value types recognized by the WRF v4.7.1 Registry parser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryValueType {
    /// Default-width Fortran integer.
    Integer,
    /// Default-width Fortran real.
    Real,
    /// Fortran logical.
    Logical,
    /// Fixed WRF Registry character storage.
    Character256,
    /// Double-precision real, including Registry's `double` alias.
    DoublePrecision,
}

impl RegistryValueType {
    /// Returns the spelling used by WRF's generated Fortran declarations.
    #[must_use]
    pub fn as_fortran(&self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Real => "real",
            Self::Logical => "logical",
            Self::Character256 => "character*256",
            Self::DoublePrecision => "doubleprecision",
        }
    }
}

impl fmt::Display for RegistryValueType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_fortran())
    }
}
