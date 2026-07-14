use wrf_compute::FieldStorage;

use crate::{
    OmegaDiagnosisCoefficients, OmegaDiagnosisGridMetrics, OmegaDiagnosisMapFactors,
    OmegaDiagnosisMasses, OmegaDiagnosisRegion, OmegaDiagnosisResult, OmegaDiagnosisVelocities,
};

/// Backend capability for diagnosing dry-air eta velocity from horizontal flow.
///
/// Implementations reproduce WRF `calc_ww_cp` while keeping field storage
/// native to the backend. The CPU implementation uses the persistent worker
/// pool; a future GPU backend can implement this capability with device kernels.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, FieldStorage, GridShape};
/// use wrf_dynamics::{
///     OmegaDiagnosisCoefficients, OmegaDiagnosisGridMetrics,
///     OmegaDiagnosisKernels, OmegaDiagnosisMapFactors,
///     OmegaDiagnosisMasses, OmegaDiagnosisRegion, OmegaDiagnosisVelocities,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(6, 6, 5)?;
/// let horizontal_shape = shape.horizontal_shape();
/// let mut omega = backend.create_field(shape, -1.0_f32)?;
/// let west_east_velocity = backend.create_field(shape, 0.0_f32)?;
/// let south_north_velocity = backend.create_field(shape, 0.0_f32)?;
/// let perturbation_mass = backend.create_field(horizontal_shape, 10.0_f32)?;
/// let base_mass = backend.create_field(horizontal_shape, 90.0_f32)?;
/// let mass_point_west_east_map_factor =
///     backend.create_field(horizontal_shape, 1.0_f32)?;
/// let west_east_momentum_south_north_map_factor =
///     backend.create_field(horizontal_shape, 1.0_f32)?;
/// let inverse_south_north_momentum_west_east_map_factor =
///     backend.create_field(horizontal_shape, 1.0_f32)?;
/// let mass_multiplier = [1.0_f32; 5];
/// let mass_offset = [0.0_f32; 5];
/// let eta_layer_thickness = [-0.25_f32; 5];
/// let region = OmegaDiagnosisRegion::try_new(
///     shape,
///     1..5,
///     1..5,
///     1..4,
///     1..5,
///     1..5,
///     1..5,
/// )?;
///
/// backend.diagnose_omega(
///     &mut omega,
///     OmegaDiagnosisVelocities::new(&west_east_velocity, &south_north_velocity),
///     OmegaDiagnosisMasses::new(&perturbation_mass, &base_mass),
///     OmegaDiagnosisMapFactors::new(
///         &mass_point_west_east_map_factor,
///         &west_east_momentum_south_north_map_factor,
///         &inverse_south_north_momentum_west_east_map_factor,
///     ),
///     OmegaDiagnosisCoefficients::new(
///         &mass_multiplier,
///         &mass_offset,
///         &eta_layer_thickness,
///     ),
///     OmegaDiagnosisGridMetrics::new(1.0, 1.0),
///     &region,
/// )?;
///
/// let west_east_points = shape.west_east_points();
/// let bottom_top_points = shape.bottom_top_points();
/// let active_index = (1 * bottom_top_points + 1) * west_east_points + 1;
/// assert_eq!(omega.values()[active_index], 0.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait OmegaDiagnosisKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Diagnoses omega through a complete vertical column.
    ///
    /// # Errors
    ///
    /// Returns a typed error if a field shape or coefficient length differs
    /// from `region`, or if backend execution fails. All validation completes
    /// before the output changes.
    #[allow(clippy::too_many_arguments)]
    fn diagnose_omega(
        &self,
        omega: &mut Self::Field,
        velocities: OmegaDiagnosisVelocities<'_, Self::Field>,
        masses: OmegaDiagnosisMasses<'_, Self::Field>,
        map_factors: OmegaDiagnosisMapFactors<'_, Self::Field>,
        coefficients: OmegaDiagnosisCoefficients<'_>,
        grid_metrics: OmegaDiagnosisGridMetrics,
        region: &OmegaDiagnosisRegion,
    ) -> OmegaDiagnosisResult<()>;
}
