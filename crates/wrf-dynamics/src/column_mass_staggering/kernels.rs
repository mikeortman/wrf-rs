use wrf_compute::FieldStorage;

use crate::{
    ColumnMassStaggeringPeriodicity, ColumnMassStaggeringRegion, ColumnMassStaggeringResult,
};

/// Backend capability for interpolating full column mass to momentum points.
///
/// This capability reproduces WRF's interior, lower-boundary, upper-boundary,
/// and two-boundary paths independently on both horizontal axes. Interior
/// momentum points average adjacent mass points. A physical-boundary point
/// copies the nearest full mass, while values outside the active output
/// rectangles remain untouched.
///
/// The [`CpuBackend`](wrf_compute::CpuBackend) implementation schedules
/// disjoint output rows on its persistent worker pool, borrows both inputs, and
/// allocates no numerical scratch. It preserves WRF's `f32` expression order.
/// Finite inputs compare by raw bits. Non-finite inputs are not pre-scanned;
/// NaN outputs compare by IEEE class because payload propagation is not a
/// portable numerical contract.
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
///     0..2,
///     0..2,
///     0..3,
///     0..3,
/// )?;
///
/// backend.stagger_column_mass(
///     &perturbation_mass,
///     &base_mass,
///     &mut west_east_mass,
///     &mut south_north_mass,
///     &region,
/// )?;
/// assert_eq!(west_east_mass.values()[0], 100.0);
/// assert_eq!(south_north_mass.values()[0], 100.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait ColumnMassStaggeringKernels {
    /// Floating-point field owned by this backend.
    type Field: FieldStorage<f32>;

    /// Places perturbation plus base mass onto both horizontal staggerings.
    ///
    /// Adjacent full-mass values are averaged at interior points. Physical
    /// boundaries use WRF's nearest-point copy rule.
    fn stagger_column_mass(
        &self,
        perturbation_mass: &Self::Field,
        base_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
    ) -> ColumnMassStaggeringResult<()>;

    /// Reproduces WRF `calc_mu_uv` for big-step perturbation and base mass.
    ///
    /// Unlike [`Self::stagger_column_mass`], physical endpoints retain WRF's
    /// duplicate-value averaging expression. This distinction matters for
    /// exceptional finite values because the intermediate addition can
    /// overflow. Periodic endpoints average the boundary mass with its halo.
    ///
    /// # Errors
    ///
    /// Returns a typed error if any field shape differs from `region`, a
    /// periodic lower endpoint lacks its preceding halo, or a CPU worker
    /// panics. Shape and halo validation finishes before either output changes.
    fn stagger_column_mass_for_big_step(
        &self,
        perturbation_mass: &Self::Field,
        base_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
        periodicity: ColumnMassStaggeringPeriodicity,
    ) -> ColumnMassStaggeringResult<()>;

    /// Reproduces WRF `calc_mu_uv_1` for already-combined full column mass.
    ///
    /// Physical and periodic endpoints retain WRF's two-addition expression
    /// order rather than simplifying duplicate operands to a copy.
    ///
    /// # Errors
    ///
    /// Returns a typed error if any field shape differs from `region`, a
    /// periodic lower endpoint lacks its preceding halo, or a CPU worker
    /// panics. Shape and halo validation finishes before either output changes.
    fn stagger_full_column_mass_for_big_step(
        &self,
        full_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
        periodicity: ColumnMassStaggeringPeriodicity,
    ) -> ColumnMassStaggeringResult<()>;
}
