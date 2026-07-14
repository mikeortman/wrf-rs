/// Controls WRF zero-gradient specified-boundary copies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecifiedBoundaryZeroGradientParameters {
    specified_zone_width: usize,
}

impl SpecifiedBoundaryZeroGradientParameters {
    /// Creates parameters for a specified zone measured in horizontal points.
    pub const fn new(specified_zone_width: usize) -> Self {
        Self {
            specified_zone_width,
        }
    }

    /// Returns the width of the outer zero-gradient zone.
    pub const fn specified_zone_width(self) -> usize {
        self.specified_zone_width
    }
}
