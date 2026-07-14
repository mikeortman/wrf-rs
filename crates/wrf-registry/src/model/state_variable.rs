use std::num::NonZeroU8;

use crate::{RegistryValueType, SourceLocation, StateDimensions, StateStaggering};

/// A parsed WRF `state` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateVariable {
    pub(crate) location: SourceLocation,
    pub(crate) value_type: RegistryValueType,
    pub(crate) name: String,
    pub(crate) dimensions: StateDimensions,
    pub(crate) use_association: Option<String>,
    pub(crate) time_levels: NonZeroU8,
    pub(crate) staggering: StateStaggering,
    pub(crate) io_specification: Option<String>,
    pub(crate) data_name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) units: Option<String>,
}

impl StateVariable {
    /// Returns the beginning of the logical `state` entry.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the declared Registry value type.
    #[must_use]
    pub const fn value_type(&self) -> &RegistryValueType {
        &self.value_type
    }

    /// Returns the state symbol.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns resolved dimensions and dimension modifiers.
    #[must_use]
    pub const fn dimensions(&self) -> &StateDimensions {
        &self.dimensions
    }

    /// Returns the optional Registry `use` association.
    #[must_use]
    pub fn use_association(&self) -> Option<&str> {
        self.use_association.as_deref()
    }

    /// Returns the positive number of generated time levels.
    #[must_use]
    pub const fn time_levels(&self) -> NonZeroU8 {
        self.time_levels
    }

    /// Returns parsed staggering and feedback flags.
    #[must_use]
    pub const fn staggering(&self) -> StateStaggering {
        self.staggering
    }

    /// Returns the optional Registry I/O specification.
    #[must_use]
    pub fn io_specification(&self) -> Option<&str> {
        self.io_specification.as_deref()
    }

    /// Returns the optional external data name.
    #[must_use]
    pub fn data_name(&self) -> Option<&str> {
        self.data_name.as_deref()
    }

    /// Returns the optional human-readable description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional unit metadata.
    #[must_use]
    pub fn units(&self) -> Option<&str> {
        self.units.as_deref()
    }
}
