use wrf_compute::FieldStorage;

use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMomentumInputs,
    AcousticHorizontalMomentumParameters, AcousticHorizontalMomentumRegion,
    AcousticHorizontalMomentumResult, AcousticHorizontalMomentumState,
};

/// Backend capability for WRF acoustic horizontal-momentum advancement.
pub trait AcousticHorizontalMomentumKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Advances U and V in WRF `advance_uv` operation order.
    ///
    /// All shape, range, neighbor, and coefficient contracts are validated
    /// before either mutable field changes. IEEE exceptional values then flow
    /// through the selected governing and boundary branches unchanged.
    fn advance_acoustic_horizontal_momentum(
        &self,
        state: AcousticHorizontalMomentumState<'_, Self::Field>,
        inputs: AcousticHorizontalMomentumInputs<'_, Self::Field>,
        parameters: AcousticHorizontalMomentumParameters,
        boundary_policy: AcousticHorizontalBoundaryPolicy,
        region: &AcousticHorizontalMomentumRegion,
    ) -> AcousticHorizontalMomentumResult<()>;
}
