/// Controls WRF flow-dependent specified scalar boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecifiedBoundaryFlowParameters {
    specified_zone_width: usize,
}

impl SpecifiedBoundaryFlowParameters {
    /// Creates parameters for a specified zone measured in horizontal points.
    pub const fn new(specified_zone_width: usize) -> Self {
        Self {
            specified_zone_width,
        }
    }

    /// Returns the width of the outer flow-dependent zone.
    pub const fn specified_zone_width(self) -> usize {
        self.specified_zone_width
    }
}
