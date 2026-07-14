use wrf_compute::CpuField;

use crate::{
    AcousticFluxAccumulationRegion, AcousticFluxCurrentFields, AcousticFluxLinearFields,
    AcousticFluxMapFactors, AcousticFluxMassFields, AcousticFluxRunningAverages,
    AcousticSubstepPhase,
};

pub(crate) struct FluxAccumulationArguments<'a, 'coefficients, 'region> {
    pub(crate) averages: AcousticFluxRunningAverages<'a, CpuField<f32>>,
    pub(crate) current: AcousticFluxCurrentFields<'a, CpuField<f32>>,
    pub(crate) linear: AcousticFluxLinearFields<'a, CpuField<f32>>,
    pub(crate) masses: AcousticFluxMassFields<'a, CpuField<f32>>,
    pub(crate) map_factors: AcousticFluxMapFactors<'a, CpuField<f32>>,
    pub(crate) column_mass_multiplier: &'coefficients [f32],
    pub(crate) column_mass_offset: &'coefficients [f32],
    pub(crate) phase: AcousticSubstepPhase,
    pub(crate) region: &'region AcousticFluxAccumulationRegion,
}
