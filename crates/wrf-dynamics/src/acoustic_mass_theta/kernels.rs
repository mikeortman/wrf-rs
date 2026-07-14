use wrf_compute::FieldStorage;

use crate::{
    AcousticMassThetaBoundaryPolicy, AcousticMassThetaDiagnostics, AcousticMassThetaInputs,
    AcousticMassThetaParameters, AcousticMassThetaRegion, AcousticMassThetaResult,
    AcousticMassThetaState,
};

/// Backend capability for acoustic column-mass, omega, and theta advancement.
pub trait AcousticMassThetaKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Advances WRF `advance_mu_t` outputs in source operation order.
    ///
    /// Shape, coefficient, range, and complete-column contracts are validated
    /// before any mutable field changes. IEEE exceptional values then flow
    /// through the numerical equations unchanged.
    fn advance_acoustic_mass_and_theta(
        &self,
        state: AcousticMassThetaState<'_, Self::Field>,
        diagnostics: AcousticMassThetaDiagnostics<'_, Self::Field>,
        inputs: AcousticMassThetaInputs<'_, Self::Field>,
        parameters: AcousticMassThetaParameters,
        boundary_policy: AcousticMassThetaBoundaryPolicy,
        region: &AcousticMassThetaRegion,
    ) -> AcousticMassThetaResult<()>;
}
