use crate::{DimensionSpecification, RuntimeConfiguration, SourceLocation, StateVariable};

/// One supported entry in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryEntry {
    /// A `dimspec` entry.
    Dimension(DimensionSpecification),
    /// A `state` entry.
    State(StateVariable),
    /// An `rconfig` entry.
    RuntimeConfiguration(RuntimeConfiguration),
}

impl RegistryEntry {
    /// Returns the beginning of this entry's logical source line.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        match self {
            Self::Dimension(dimension) => dimension.location(),
            Self::State(state) => state.location(),
            Self::RuntimeConfiguration(configuration) => configuration.location(),
        }
    }
}
