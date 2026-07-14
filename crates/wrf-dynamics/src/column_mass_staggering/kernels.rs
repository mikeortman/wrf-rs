use wrf_compute::FieldStorage;

use crate::{ColumnMassStaggeringRegion, ColumnMassStaggeringResult};

/// Backend capability for interpolating full column mass to momentum points.
///
/// The current capability represents WRF's interior-tile path, where each
/// momentum point has a preceding mass-grid neighbor. Physical-boundary copy
/// branches will be added before the upstream routine is considered complete.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{ColumnMassStaggeringKernels, ColumnMassStaggeringRegion};
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(3, 3, 1)?;
/// let perturbation_mass = backend.create_field(shape, 2.0_f32)?;
/// let base_mass = backend.create_field(shape, 98.0_f32)?;
/// let mut west_east_mass = backend.create_field(shape, -1.0_f32)?;
/// let mut south_north_mass = backend.create_field(shape, -1.0_f32)?;
/// let region = ColumnMassStaggeringRegion::try_new(
///     shape,
///     1..3,
///     1..3,
///     1..3,
///     1..3,
/// )?;
///
/// backend.stagger_column_mass(
///     &perturbation_mass,
///     &base_mass,
///     &mut west_east_mass,
///     &mut south_north_mass,
///     &region,
/// )?;
/// assert_eq!(west_east_mass.values()[4], 100.0);
/// assert_eq!(south_north_mass.values()[4], 100.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait ColumnMassStaggeringKernels {
    /// Floating-point field owned by this backend.
    type Field: FieldStorage<f32>;

    /// Averages perturbation plus base mass onto both horizontal staggerings.
    fn stagger_column_mass(
        &self,
        perturbation_mass: &Self::Field,
        base_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
    ) -> ColumnMassStaggeringResult<()>;
}
