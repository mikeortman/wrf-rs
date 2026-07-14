use wrf_compute::FieldStorage;

use crate::{
    SpecifiedBoundaryGeopotentialInputs, SpecifiedBoundaryGeopotentialResult,
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

/// Backend capability for mass-normalized geopotential boundary updates.
///
/// Implementations reproduce WRF v4.7.1 `spec_bdyupdate_ph` while retaining
/// native field storage for CPU and future accelerator backends.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFieldLocation, SpecifiedBoundaryGeopotentialInputs,
///     SpecifiedBoundaryGeopotentialKernels, SpecifiedBoundaryUpdateParameters,
///     SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(6, 6, 6)?;
/// let mut geopotential = backend.create_field(shape, 20.0_f32)?;
/// let saved = backend.create_field(shape, 19.0_f32)?;
/// let tendency = backend.create_field(shape, 0.2_f32)?;
/// let mass_tendency = backend.create_field(shape.horizontal_shape(), 0.1_f32)?;
/// let current_mass = backend.create_field(shape.horizontal_shape(), 10.0_f32)?;
/// let multiplier = vec![0.4_f32; 6];
/// let offset = vec![2.0_f32; 6];
/// let region = SpecifiedBoundaryUpdateRegion::try_new(
///     shape,
///     SpecifiedBoundaryFieldLocation::FullLevel,
///     1..5,
///     1..5,
///     1..5,
///     1..6,
///     1..6,
///     1..6,
/// )?;
/// backend.update_specified_boundary_geopotential(
///     &mut geopotential,
///     SpecifiedBoundaryGeopotentialInputs::new(
///         &saved,
///         &tendency,
///         &mass_tendency,
///         &current_mass,
///         &multiplier,
///         &offset,
///     ),
///     SpecifiedBoundaryUpdateParameters::new(0.25, 2),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryGeopotentialKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Applies the supplied tendency and column-mass normalization in the zone.
    ///
    /// Validation completes before mutation. A zero-width zone is an exact
    /// no-op after validation, and inactive storage remains unchanged.
    ///
    /// # Errors
    ///
    /// Returns a typed error for incompatible field shapes, incomplete
    /// vertical coefficients, or backend execution failure.
    fn update_specified_boundary_geopotential(
        &self,
        geopotential: &mut Self::Field,
        inputs: SpecifiedBoundaryGeopotentialInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryUpdateParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryGeopotentialResult<()>;
}
