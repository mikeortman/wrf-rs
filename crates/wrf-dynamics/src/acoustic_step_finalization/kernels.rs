use wrf_compute::FieldStorage;

use crate::{
    AcousticStepFinalizationCoefficients, AcousticStepFinalizationControls,
    AcousticStepFinalizationMapFactors, AcousticStepFinalizationMasses,
    AcousticStepFinalizationRegion, AcousticStepFinalizationResult,
    AcousticStepFinalizationSavedState, AcousticStepFinalizationState,
};

/// Backend capability for WRF `small_step_finish` field reconstruction.
pub trait AcousticStepFinalizationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Reconstructs uncoupled prognostic fields after acoustic substeps.
    ///
    /// WRF arguments that the routine never reads (`*_1`, `c3*`, `c4*`, and
    /// three map-factor roles) are deliberately absent. Every live field,
    /// coefficient, and region contract is checked before the first mutation.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or persistent-worker failure. Validation
    /// failures leave all mutable fields unchanged.
    #[allow(clippy::too_many_arguments)]
    fn finalize_acoustic_step(
        &self,
        state: AcousticStepFinalizationState<'_, Self::Field>,
        masses: AcousticStepFinalizationMasses<'_, Self::Field>,
        saved: AcousticStepFinalizationSavedState<'_, Self::Field>,
        map_factors: AcousticStepFinalizationMapFactors<'_, Self::Field>,
        coefficients: AcousticStepFinalizationCoefficients<'_>,
        controls: AcousticStepFinalizationControls,
        region: &AcousticStepFinalizationRegion,
    ) -> AcousticStepFinalizationResult<()>;
}
