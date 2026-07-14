use crate::WrfDimensionName;

/// One dimension in a WRF NetCDF schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrfDimension {
    name: WrfDimensionName,
    length: usize,
    is_unlimited: bool,
}

impl WrfDimension {
    pub(crate) const fn fixed(name: WrfDimensionName, length: usize) -> Self {
        Self {
            name,
            length,
            is_unlimited: false,
        }
    }

    pub(crate) const fn unlimited(name: WrfDimensionName, length: usize) -> Self {
        Self {
            name,
            length,
            is_unlimited: true,
        }
    }

    /// Returns the exact WRF dimension name.
    pub const fn name(&self) -> WrfDimensionName {
        self.name
    }

    /// Returns the current dimension length.
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Reports whether this is the record dimension.
    pub const fn is_unlimited(&self) -> bool {
        self.is_unlimited
    }
}
