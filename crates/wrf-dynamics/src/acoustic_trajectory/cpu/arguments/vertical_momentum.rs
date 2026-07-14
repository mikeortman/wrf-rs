use wrf_compute::CpuField;

use crate::{
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
    AcousticVerticalInputs, AcousticVerticalParameters, AcousticVerticalRegion,
    AcousticVerticalState, AcousticVerticalWorkspace, VerticalAcousticTopBoundary,
};

pub(crate) struct VerticalMomentumArguments<'a, 'region> {
    pub(crate) state: AcousticVerticalState<'a, CpuField<f32>>,
    pub(crate) workspace: AcousticVerticalWorkspace<'a, CpuField<f32>>,
    pub(crate) inputs: AcousticVerticalInputs<'a, CpuField<f32>>,
    pub(crate) parameters: AcousticVerticalParameters,
    pub(crate) boundary_policy: AcousticVerticalBoundaryPolicy,
    pub(crate) top_boundary: VerticalAcousticTopBoundary,
    pub(crate) vertical_advection: AcousticVerticalAdvection,
    pub(crate) damping: AcousticVerticalDamping,
    pub(crate) region: &'region AcousticVerticalRegion,
}
