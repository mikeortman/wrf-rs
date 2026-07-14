use crate::{RegistryValueType, SourceLocation};

/// Scalar or expression-sized storage for a runtime configuration entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigurationEntryCount {
    /// One value, represented by `1` or `-` in the source.
    Scalar,
    /// A dimension expression such as `max_domains`.
    Expression(String),
}

impl ConfigurationEntryCount {
    /// Returns the expression expected by WRF's Fortran generator.
    #[must_use]
    pub fn as_registry_expression(&self) -> &str {
        match self {
            Self::Scalar => "1",
            Self::Expression(expression) => expression,
        }
    }
}

/// A parsed WRF `rconfig` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfiguration {
    pub(crate) location: SourceLocation,
    pub(crate) value_type: RegistryValueType,
    pub(crate) name: String,
    pub(crate) how_set: Option<String>,
    pub(crate) entry_count: ConfigurationEntryCount,
    pub(crate) default_value: Option<String>,
    pub(crate) io_specification: Option<String>,
    pub(crate) data_name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) units: Option<String>,
}

impl RuntimeConfiguration {
    /// Returns the beginning of the logical `rconfig` entry.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the declared Registry value type.
    #[must_use]
    pub const fn value_type(&self) -> &RegistryValueType {
        &self.value_type
    }

    /// Returns the runtime-configuration symbol.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the raw setting mechanism, such as `namelist,time_control`.
    #[must_use]
    pub fn how_set(&self) -> Option<&str> {
        self.how_set.as_deref()
    }

    /// Returns whether storage is scalar or expression-sized.
    #[must_use]
    pub const fn entry_count(&self) -> &ConfigurationEntryCount {
        &self.entry_count
    }

    /// Returns the optional source default without added Fortran quoting.
    #[must_use]
    pub fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
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

    /// Returns the section when this entry is set through a namelist.
    #[must_use]
    pub fn namelist_section(&self) -> Option<&str> {
        let how_set = self.how_set.as_deref()?;
        let (mechanism, section) = how_set.split_once(',')?;
        (mechanism == "namelist").then_some(section)
    }
}
