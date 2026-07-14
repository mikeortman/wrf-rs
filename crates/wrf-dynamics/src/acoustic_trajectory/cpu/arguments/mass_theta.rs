use wrf_compute::CpuField;

use crate::{
    AcousticMassThetaBoundaryPolicy, AcousticMassThetaDiagnostics, AcousticMassThetaInputs,
    AcousticMassThetaParameters, AcousticMassThetaRegion, AcousticMassThetaState,
};

pub(crate) struct MassThetaArguments<'a, 'region> {
    pub(crate) state: AcousticMassThetaState<'a, CpuField<f32>>,
    pub(crate) diagnostics: AcousticMassThetaDiagnostics<'a, CpuField<f32>>,
    pub(crate) inputs: AcousticMassThetaInputs<'a, CpuField<f32>>,
    pub(crate) parameters: AcousticMassThetaParameters,
    pub(crate) boundary_policy: AcousticMassThetaBoundaryPolicy,
    pub(crate) region: &'region AcousticMassThetaRegion,
}
