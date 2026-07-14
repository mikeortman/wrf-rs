/// Processor orientation encoded by a state dimension modifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessorOrientation {
    /// All X coordinates reside on a processor.
    X,
    /// All Y coordinates reside on a processor.
    Y,
    /// All Z coordinates reside on a processor; this is WRF's default.
    Z,
}

/// Resolved state dimensions and their WRF Registry modifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateDimensions {
    pub(crate) names: Vec<String>,
    pub(crate) subgrid_positions: Vec<usize>,
    pub(crate) processor_orientation: ProcessorOrientation,
    pub(crate) is_boundary_array: bool,
    pub(crate) is_scalar_array_member: bool,
    pub(crate) has_scalar_array_tendencies: bool,
}

impl StateDimensions {
    /// Returns resolved dimension symbols in Registry memory order.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Returns zero-based positions marked as subgrid dimensions.
    #[must_use]
    pub fn subgrid_positions(&self) -> &[usize] {
        &self.subgrid_positions
    }

    /// Returns the processor-local coordinate orientation.
    #[must_use]
    pub const fn processor_orientation(&self) -> ProcessorOrientation {
        self.processor_orientation
    }

    /// Returns whether WRF creates boundary and boundary-tendency arrays.
    #[must_use]
    pub const fn is_boundary_array(&self) -> bool {
        self.is_boundary_array
    }

    /// Returns whether the state belongs to a four-dimensional scalar array.
    #[must_use]
    pub const fn is_scalar_array_member(&self) -> bool {
        self.is_scalar_array_member
    }

    /// Returns whether that scalar array also has generated tendencies.
    #[must_use]
    pub const fn has_scalar_array_tendencies(&self) -> bool {
        self.has_scalar_array_tendencies
    }
}
