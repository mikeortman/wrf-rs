use wrf_compute::CpuField;

use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMomentumInputs,
    AcousticHorizontalMomentumParameters, AcousticHorizontalMomentumRegion,
    AcousticHorizontalMomentumState,
};

pub(crate) struct HorizontalMomentumArguments<'a, 'region> {
    pub(crate) state: AcousticHorizontalMomentumState<'a, CpuField<f32>>,
    pub(crate) inputs: AcousticHorizontalMomentumInputs<'a, CpuField<f32>>,
    pub(crate) parameters: AcousticHorizontalMomentumParameters,
    pub(crate) boundary_policy: AcousticHorizontalBoundaryPolicy,
    pub(crate) region: &'region AcousticHorizontalMomentumRegion,
}
