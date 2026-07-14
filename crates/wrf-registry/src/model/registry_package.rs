use crate::{PackageCondition, PackageVariableGroup, SourceLocation};

/// A parsed generic WRF Registry `package` declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryPackage {
    pub(crate) location: SourceLocation,
    pub(crate) name: String,
    pub(crate) condition: PackageCondition,
    pub(crate) variable_groups: Vec<PackageVariableGroup>,
}

impl RegistryPackage {
    /// Returns the beginning of this package's logical source line.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the generated Registry package constant name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the signed runtime-configuration equality condition.
    #[must_use]
    pub const fn condition(&self) -> &PackageCondition {
        &self.condition
    }

    /// Returns generic variable groups in source order.
    #[must_use]
    pub fn variable_groups(&self) -> &[PackageVariableGroup] {
        &self.variable_groups
    }
}
