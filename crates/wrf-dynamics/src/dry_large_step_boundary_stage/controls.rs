use crate::{SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity};

/// Scalar controls shared by relaxation and boundary assignment.
///
/// The Runge–Kutta substep is not a control here: it is carried by the typed
/// stage mode so relaxation activity and assembly accumulation cannot diverge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DryLargeStepBoundaryStageControls {
    pub(crate) boundary_parameters: SpecifiedBoundaryTendencyParameters,
    pub(crate) west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
}

impl DryLargeStepBoundaryStageControls {
    /// Creates controls for one large-timestep Runge–Kutta stage.
    pub const fn new(
        boundary_parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    ) -> Self {
        Self {
            boundary_parameters,
            west_east_periodicity,
        }
    }
}
