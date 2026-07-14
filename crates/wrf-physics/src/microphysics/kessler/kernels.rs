use wrf_compute::FieldStorage;

use crate::{
    KesslerMicrophysicsFields, KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
    KesslerMicrophysicsResult,
};

/// Backend capability for WRF Kessler warm-rain microphysics.
///
/// Fields and reusable scratch remain backend-owned so a future GPU backend can
/// keep both device-resident. The CPU implementation schedules independent
/// south-north rows across its persistent worker pool by default.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_physics::{
///     KesslerMicrophysicsFields, KesslerMicrophysicsKernels,
///     KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(2, 2, 2)?;
/// let mut potential_temperature = backend.create_field(shape, 280.0_f32)?;
/// let mut water_vapor = backend.create_field(shape, 0.005_f32)?;
/// let mut cloud_water = backend.create_field(shape, 0.001_f32)?;
/// let mut rain_water = backend.create_field(shape, 0.0005_f32)?;
/// let density = backend.create_field(shape, 1.0_f32)?;
/// let exner = backend.create_field(shape, 0.95_f32)?;
/// let mut height = backend.create_field(shape, 50.0_f32)?;
/// for (index, value) in height.values_mut().iter_mut().enumerate() {
///     if (index / shape.west_east_points()) % shape.bottom_top_points() == 1 {
///         *value = 200.0;
///     }
/// }
/// let layer_thickness = backend.create_field(shape, 150.0_f32)?;
/// let mut accumulated_precipitation =
///     backend.create_field(shape.horizontal_shape(), 0.0_f32)?;
/// let mut step_precipitation =
///     backend.create_field(shape.horizontal_shape(), 0.0_f32)?;
/// let region = KesslerMicrophysicsRegion::try_new(shape, 0..2, 0..2, 0..2)?;
/// let parameters = KesslerMicrophysicsParameters::try_from_wrf_defaults(30.0)?;
/// let mut workspace = backend.create_kessler_microphysics_workspace(&region)?;
///
/// backend.apply_kessler_microphysics(
///     KesslerMicrophysicsFields::new(
///         &mut potential_temperature,
///         &mut water_vapor,
///         &mut cloud_water,
///         &mut rain_water,
///         &density,
///         &exner,
///         &height,
///         &layer_thickness,
///         &mut accumulated_precipitation,
///         &mut step_precipitation,
///     ),
///     parameters,
///     &region,
///     &mut workspace,
/// )?;
/// # Ok(())
/// # }
/// ```
pub trait KesslerMicrophysicsKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Backend-native reusable sedimentation workspace.
    type Workspace;

    /// Allocates reusable scratch for a validated region.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend cannot allocate the required workspace.
    fn create_kessler_microphysics_workspace(
        &self,
        region: &KesslerMicrophysicsRegion,
    ) -> KesslerMicrophysicsResult<Self::Workspace>;

    /// Applies one Kessler sedimentation and warm-rain conversion update.
    ///
    /// # Errors
    ///
    /// Returns an error before field mutation if shapes or workspace do not
    /// match the region, or if parallel CPU execution fails.
    fn apply_kessler_microphysics(
        &self,
        fields: KesslerMicrophysicsFields<'_, Self::Field>,
        parameters: KesslerMicrophysicsParameters,
        region: &KesslerMicrophysicsRegion,
        workspace: &mut Self::Workspace,
    ) -> KesslerMicrophysicsResult<()>;
}
