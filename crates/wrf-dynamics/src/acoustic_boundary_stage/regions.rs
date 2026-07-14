use crate::{AcousticTrajectoryRegions, PhysicalBoundaryRegion, SpecifiedBoundaryUpdateRegion};

/// Validated numerical and lateral-boundary regions for one acoustic stage.
#[derive(Clone, Copy, Debug)]
pub struct AcousticBoundaryStageRegions<'a> {
    pub(crate) trajectory: AcousticTrajectoryRegions<'a>,
    pub(crate) physical: &'a PhysicalBoundaryRegion,
    pub(crate) west_east_momentum: &'a SpecifiedBoundaryUpdateRegion,
    pub(crate) south_north_momentum: &'a SpecifiedBoundaryUpdateRegion,
    pub(crate) potential_temperature: &'a SpecifiedBoundaryUpdateRegion,
    pub(crate) column_mass: &'a SpecifiedBoundaryUpdateRegion,
    pub(crate) full_level: &'a SpecifiedBoundaryUpdateRegion,
}

impl<'a> AcousticBoundaryStageRegions<'a> {
    /// Groups existing typed regions without recomputing their ranges.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        trajectory: AcousticTrajectoryRegions<'a>,
        physical: &'a PhysicalBoundaryRegion,
        west_east_momentum: &'a SpecifiedBoundaryUpdateRegion,
        south_north_momentum: &'a SpecifiedBoundaryUpdateRegion,
        potential_temperature: &'a SpecifiedBoundaryUpdateRegion,
        column_mass: &'a SpecifiedBoundaryUpdateRegion,
        full_level: &'a SpecifiedBoundaryUpdateRegion,
    ) -> Self {
        Self {
            trajectory,
            physical,
            west_east_momentum,
            south_north_momentum,
            potential_temperature,
            column_mass,
            full_level,
        }
    }
}
