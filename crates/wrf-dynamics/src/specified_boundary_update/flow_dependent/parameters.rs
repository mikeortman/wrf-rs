/// Controls WRF flow-dependent specified scalar boundaries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecifiedBoundaryFlowParameters {
    specified_zone_width: usize,
    inflow_policy: crate::SpecifiedBoundaryInflowPolicy,
}

impl SpecifiedBoundaryFlowParameters {
    /// Creates parameters for a specified zone measured in horizontal points.
    pub const fn new(specified_zone_width: usize) -> Self {
        Self {
            specified_zone_width,
            inflow_policy: crate::SpecifiedBoundaryInflowPolicy::Zero,
        }
    }

    /// Creates parameters with an explicit WRF-compatible inflow action.
    pub const fn with_inflow_policy(
        specified_zone_width: usize,
        inflow_policy: crate::SpecifiedBoundaryInflowPolicy,
    ) -> Self {
        Self {
            specified_zone_width,
            inflow_policy,
        }
    }

    /// Returns the width of the outer flow-dependent zone.
    pub const fn specified_zone_width(self) -> usize {
        self.specified_zone_width
    }

    /// Returns the value action used when velocity identifies inflow.
    pub const fn inflow_policy(self) -> crate::SpecifiedBoundaryInflowPolicy {
        self.inflow_policy
    }
}
