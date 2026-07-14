use crate::WrfAttributeValue;

/// One named global or variable NetCDF attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct WrfAttribute {
    name: String,
    value: WrfAttributeValue,
}

impl WrfAttribute {
    /// Creates a typed attribute. Schema constructors validate names before I/O.
    pub fn new(name: impl Into<String>, value: WrfAttributeValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Returns the attribute name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the typed attribute value.
    pub const fn value(&self) -> &WrfAttributeValue {
        &self.value
    }
}
