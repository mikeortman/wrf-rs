mod horizontal;
mod validation;
mod volume;

use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    AcousticStepPreparationCoefficients, AcousticStepPreparationColumnMassTimeLevels,
    AcousticStepPreparationDiagnosticInputs, AcousticStepPreparationError,
    AcousticStepPreparationKernels, AcousticStepPreparationMapFactors,
    AcousticStepPreparationMassInputs, AcousticStepPreparationMassOutputs,
    AcousticStepPreparationPhase, AcousticStepPreparationRegion, AcousticStepPreparationResult,
    AcousticStepPreparationSavedOutputs, AcousticStepPreparationVolumeTimeLevels,
};

impl AcousticStepPreparationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn prepare_acoustic_step(
        &self,
        mut volume_time_levels: AcousticStepPreparationVolumeTimeLevels<'_, Self::Field>,
        mut column_mass_time_levels: AcousticStepPreparationColumnMassTimeLevels<'_, Self::Field>,
        mut saved_outputs: AcousticStepPreparationSavedOutputs<'_, Self::Field>,
        mut mass_outputs: AcousticStepPreparationMassOutputs<'_, Self::Field>,
        mass_inputs: AcousticStepPreparationMassInputs<'_, Self::Field>,
        diagnostics: AcousticStepPreparationDiagnosticInputs<'_, Self::Field>,
        map_factors: AcousticStepPreparationMapFactors<'_, Self::Field>,
        coefficients: AcousticStepPreparationCoefficients<'_>,
        phase: AcousticStepPreparationPhase,
        region: &AcousticStepPreparationRegion,
    ) -> AcousticStepPreparationResult<()> {
        validation::validate_operation(
            &volume_time_levels,
            &column_mass_time_levels,
            &saved_outputs,
            &mass_outputs,
            &mass_inputs,
            &diagnostics,
            &map_factors,
            coefficients,
            phase,
            region,
        )?;

        if phase.switches_time_levels() {
            volume::switch_time_levels(self, &mut volume_time_levels, region)?;
        }
        horizontal::prepare_column_masses(
            self,
            &mut column_mass_time_levels,
            &mut saved_outputs,
            &mut mass_outputs,
            &mass_inputs,
            phase,
            region,
        )?;
        volume::prepare_coupled_fields(
            self,
            &mut volume_time_levels,
            &mut saved_outputs,
            &mass_outputs,
            &mass_inputs,
            &diagnostics,
            &map_factors,
            coefficients,
            region,
        )
    }
}

pub(super) fn map_parallel_error(
    error: ParallelExecutionError<Infallible>,
) -> AcousticStepPreparationError {
    match error {
        ParallelExecutionError::WorkerPanicked => AcousticStepPreparationError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            AcousticStepPreparationError::SchedulerContractViolated
        }
        ParallelExecutionError::Kernel(unreachable) => match unreachable {},
    }
}
