use crate::SourceLocation;

/// Ordered variables associated with one generic Registry package group.
///
/// A source field such as `moist:qv,qc,qr` becomes one group. Both the group
/// order and member order are retained exactly for WRF packed-index parity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageVariableGroup {
    pub(crate) location: SourceLocation,
    pub(crate) scalar_array_name: String,
    pub(crate) members: Vec<String>,
}

impl PackageVariableGroup {
    /// Returns the physical location of the package declaration.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the associated four-dimensional scalar-array name.
    #[must_use]
    pub fn scalar_array_name(&self) -> &str {
        &self.scalar_array_name
    }

    /// Returns member names in their Registry source order.
    #[must_use]
    pub fn members(&self) -> &[String] {
        &self.members
    }
}
