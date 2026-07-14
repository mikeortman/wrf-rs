use crate::{
    DryTendencyAssemblyPhase, SpecifiedBoundaryTendencyParameters,
    SpecifiedBoundaryWestEastPeriodicity,
};

/// Scalar controls shared by the two consecutive WRF routines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DryTendencyBoundaryStageControls {
    pub(crate) phase: DryTendencyAssemblyPhase,
    pub(crate) boundary_parameters: SpecifiedBoundaryTendencyParameters,
    pub(crate) west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
}

impl DryTendencyBoundaryStageControls {
    /// Creates controls for one large-timestep Runge–Kutta stage.
    pub const fn new(
        phase: DryTendencyAssemblyPhase,
        boundary_parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    ) -> Self {
        Self {
            phase,
            boundary_parameters,
            west_east_periodicity,
        }
    }
}
