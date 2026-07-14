use wrf_compute::FieldStorage;

use crate::{
    AcousticFluxAccumulationRegion, AcousticFluxAccumulationResult, AcousticFluxCurrentFields,
    AcousticFluxLinearFields, AcousticFluxMapFactors, AcousticFluxMassFields,
    AcousticFluxRunningAverages, AcousticSubstepPhase,
};

/// Backend capability for WRF acoustic mass-flux accumulation.
pub trait AcousticFluxAccumulationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Accumulates one substep and finalizes time averages on the last substep.
    ///
    /// All shapes and coefficients are validated before any output changes.
    #[allow(clippy::too_many_arguments)]
    fn accumulate_acoustic_fluxes(
        &self,
        averages: AcousticFluxRunningAverages<'_, Self::Field>,
        current: AcousticFluxCurrentFields<'_, Self::Field>,
        linear: AcousticFluxLinearFields<'_, Self::Field>,
        masses: AcousticFluxMassFields<'_, Self::Field>,
        map_factors: AcousticFluxMapFactors<'_, Self::Field>,
        column_mass_multiplier: &[f32],
        column_mass_offset: &[f32],
        phase: AcousticSubstepPhase,
        region: &AcousticFluxAccumulationRegion,
    ) -> AcousticFluxAccumulationResult<()>;
}
