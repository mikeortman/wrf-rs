use wrf_compute::CpuField;

use crate::{
    AcousticStepPreparationCoefficients, AcousticStepPreparationColumnMassTimeLevels,
    AcousticStepPreparationDiagnosticInputs, AcousticStepPreparationMapFactors,
    AcousticStepPreparationMassInputs, AcousticStepPreparationMassOutputs,
    AcousticStepPreparationPhase, AcousticStepPreparationRegion,
    AcousticStepPreparationSavedOutputs, AcousticStepPreparationVolumeTimeLevels,
};

pub(crate) struct PreparationArguments<'a, 'coefficients, 'region> {
    pub(crate) volume: AcousticStepPreparationVolumeTimeLevels<'a, CpuField<f32>>,
    pub(crate) column_mass: AcousticStepPreparationColumnMassTimeLevels<'a, CpuField<f32>>,
    pub(crate) saved: AcousticStepPreparationSavedOutputs<'a, CpuField<f32>>,
    pub(crate) mass_outputs: AcousticStepPreparationMassOutputs<'a, CpuField<f32>>,
    pub(crate) mass_inputs: AcousticStepPreparationMassInputs<'a, CpuField<f32>>,
    pub(crate) diagnostics: AcousticStepPreparationDiagnosticInputs<'a, CpuField<f32>>,
    pub(crate) maps: AcousticStepPreparationMapFactors<'a, CpuField<f32>>,
    pub(crate) coefficients: AcousticStepPreparationCoefficients<'coefficients>,
    pub(crate) phase: AcousticStepPreparationPhase,
    pub(crate) region: &'region AcousticStepPreparationRegion,
}
