/// Scalar controls for WRF specified-boundary relaxation forcing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecifiedBoundaryRelaxationParameters {
    pub(crate) boundary_width: usize,
    pub(crate) specified_zone_width: usize,
    pub(crate) relaxation_zone_width: usize,
    pub(crate) boundary_time_offset: f32,
}

impl SpecifiedBoundaryRelaxationParameters {
    /// Captures stored width, fixed and relaxed zones, and boundary time offset.
    pub const fn new(
        boundary_width: usize,
        specified_zone_width: usize,
        relaxation_zone_width: usize,
        boundary_time_offset: f32,
    ) -> Self {
        Self {
            boundary_width,
            specified_zone_width,
            relaxation_zone_width,
            boundary_time_offset,
        }
    }

    /// Returns the number of stored points normal to each boundary.
    pub const fn boundary_width(self) -> usize {
        self.boundary_width
    }

    /// Returns the outer fixed zone excluded from relaxation.
    pub const fn specified_zone_width(self) -> usize {
        self.specified_zone_width
    }

    /// Returns the inner edge of the complete relaxation zone.
    pub const fn relaxation_zone_width(self) -> usize {
        self.relaxation_zone_width
    }

    /// Returns the offset from the boundary-file state to the requested time.
    pub const fn boundary_time_offset(self) -> f32 {
        self.boundary_time_offset
    }
}
