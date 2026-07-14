mod damping;
mod hydrostatic;
mod nonhydrostatic;
mod validation;

use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    AcousticPressureCoefficients, AcousticPressureDampingPhase, AcousticPressureError,
    AcousticPressureKernels, AcousticPressureMasses, AcousticPressureMode,
    AcousticPressureParameters, AcousticPressureRegion, AcousticPressureResult,
    AcousticPressureState, AcousticPressureThermodynamics, AcousticPressureVerticalMetrics,
};

impl AcousticPressureKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn update_acoustic_pressure(
        &self,
        mut state: AcousticPressureState<'_, Self::Field>,
        thermodynamics: AcousticPressureThermodynamics<'_, Self::Field>,
        masses: AcousticPressureMasses<'_, Self::Field>,
        coefficients: AcousticPressureCoefficients<'_>,
        vertical_metrics: AcousticPressureVerticalMetrics<'_>,
        parameters: AcousticPressureParameters,
        mode: AcousticPressureMode,
        damping_phase: AcousticPressureDampingPhase,
        region: &AcousticPressureRegion,
    ) -> AcousticPressureResult<()> {
        validation::validate_operation(
            &state,
            &thermodynamics,
            &masses,
            coefficients,
            vertical_metrics,
            region,
        )?;
        match mode {
            AcousticPressureMode::Nonhydrostatic => nonhydrostatic::diagnose(
                self,
                &mut state,
                &thermodynamics,
                &masses,
                coefficients,
                vertical_metrics,
                parameters,
                region,
            )?,
            AcousticPressureMode::Hydrostatic => hydrostatic::diagnose(
                self,
                &mut state,
                &thermodynamics,
                &masses,
                coefficients,
                vertical_metrics,
                parameters,
                region,
            )?,
        }
        damping::apply(self, &mut state, parameters, damping_phase, region)
    }
}

pub(super) fn map_parallel_error(
    error: ParallelExecutionError<Infallible>,
) -> AcousticPressureError {
    match error {
        ParallelExecutionError::WorkerPanicked => AcousticPressureError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            AcousticPressureError::SchedulerContractViolated
        }
        ParallelExecutionError::Kernel(unreachable) => match unreachable {},
    }
}
