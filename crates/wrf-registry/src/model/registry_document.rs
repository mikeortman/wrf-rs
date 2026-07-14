use std::sync::Arc;

use crate::{DimensionSpecification, RegistryEntry, RuntimeConfiguration, StateVariable};

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
}
