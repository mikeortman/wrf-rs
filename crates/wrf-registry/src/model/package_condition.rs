use crate::SourceLocation;

/// Signed integer-equality condition controlling one Registry package.
///
/// WRF spells this field as `configuration_name==choice`; package selection
/// compares it with one domain's runtime-configuration value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageCondition {
    pub(crate) location: SourceLocation,
    pub(crate) configuration_name: String,
    pub(crate) choice: i32,
}

impl PackageCondition {
    /// Returns the physical location of the package declaration.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the runtime-configuration symbol on the left side of `==`.
    #[must_use]
    pub fn configuration_name(&self) -> &str {
        &self.configuration_name
    }

    /// Returns the signed integer required to activate the package.
    #[must_use]
    pub const fn choice(&self) -> i32 {
        self.choice
    }
}
