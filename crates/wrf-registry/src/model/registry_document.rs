use std::sync::Arc;

use crate::{
    DimensionSpecification, RegistryEntry, RegistryPackage, RegistryResolutionResult,
    ResolvedScalarArrayLayout, RuntimeConfiguration, RuntimeConfigurationChoice, StateVariable,
};

/// Typed Registry entries parsed from one source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryDocument {
    pub(crate) source_name: Arc<str>,
    pub(crate) entries: Vec<RegistryEntry>,
}

impl RegistryDocument {
    /// Returns the source name supplied to [`crate::RegistryParser`].
    #[must_use]
    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    /// Returns every supported entry in source order.
    #[must_use]
    pub fn entries(&self) -> &[RegistryEntry] {
        &self.entries
    }

    /// Iterates over parsed dimension specifications in source order.
    pub fn dimensions(&self) -> impl Iterator<Item = &DimensionSpecification> {
        self.entries.iter().filter_map(|entry| match entry {
            RegistryEntry::Dimension(dimension) => Some(dimension),
            _ => None,
        })
    }

    /// Iterates over parsed state variables in source order.
    pub fn state_variables(&self) -> impl Iterator<Item = &StateVariable> {
        self.entries.iter().filter_map(|entry| match entry {
            RegistryEntry::State(state) => Some(state),
            _ => None,
        })
    }

    /// Iterates over parsed runtime configurations in source order.
    pub fn runtime_configurations(&self) -> impl Iterator<Item = &RuntimeConfiguration> {
        self.entries.iter().filter_map(|entry| match entry {
            RegistryEntry::RuntimeConfiguration(configuration) => Some(configuration),
            _ => None,
        })
    }

    /// Iterates over parsed package declarations in source order.
    pub fn packages(&self) -> impl Iterator<Item = &RegistryPackage> {
        self.entries.iter().filter_map(|entry| match entry {
            RegistryEntry::Package(package) => Some(package),
            _ => None,
        })
    }

    /// Resolves active four-dimensional scalar arrays for one domain.
    ///
    /// Every package whose signed integer condition matches is applied in
    /// Registry source order. Repeated member activation reuses the first
    /// packed position, matching WRF's generated index tables.
    ///
    /// # Errors
    ///
    /// Returns a typed error for duplicate or unknown runtime choices,
    /// malformed cross-entry associations, missing reserved placeholders, or
    /// package references to unknown scalar arrays and members. Resolution is
    /// failure-atomic: no partial layout is returned with an error.
    pub fn resolve_scalar_array_layouts(
        &self,
        configuration_choices: &[RuntimeConfigurationChoice],
    ) -> RegistryResolutionResult<Vec<ResolvedScalarArrayLayout>> {
        crate::scalar_layout::ScalarArrayLayoutResolver::resolve(self, configuration_choices)
    }
}
