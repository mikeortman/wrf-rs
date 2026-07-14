use wrf_compute::FieldStorage;

use crate::{
    MomentumCouplingCoefficients, MomentumCouplingMapFactors, MomentumCouplingMasses,
    MomentumCouplingOutputs, MomentumCouplingRegion, MomentumCouplingResult,
    MomentumCouplingVelocities,
};

/// Backend capability for coupling C-grid velocities to dry-air column mass.
///
/// Implementations reproduce WRF `couple_momentum` while keeping native field
/// storage behind the backend. The CPU path uses the persistent worker pool;
/// future GPU backends can provide native kernels without exposing host slices.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, FieldStorage, GridShape};
/// use wrf_dynamics::{
///     MomentumCouplingCoefficients, MomentumCouplingKernels,
///     MomentumCouplingMapFactors, MomentumCouplingMasses,
///     MomentumCouplingOutputs, MomentumCouplingRegion,
///     MomentumCouplingVelocities,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(4, 4, 4)?;
/// let horizontal_shape = shape.horizontal_shape();
/// let mut west_east_output = backend.create_field(shape, -1.0_f32)?;
/// let mut south_north_output = backend.create_field(shape, -1.0_f32)?;
/// let mut vertical_output = backend.create_field(shape, -1.0_f32)?;
/// let west_east_velocity = backend.create_field(shape, 2.0_f32)?;
/// let south_north_velocity = backend.create_field(shape, 2.0_f32)?;
/// let vertical_velocity = backend.create_field(shape, 2.0_f32)?;
/// let west_east_mass = backend.create_field(horizontal_shape, 10.0_f32)?;
/// let south_north_mass = backend.create_field(horizontal_shape, 10.0_f32)?;
/// let mass_point_mass = backend.create_field(horizontal_shape, 10.0_f32)?;
/// let west_east_map_factor = backend.create_field(horizontal_shape, 2.0_f32)?;
/// let inverse_south_north_map_factor =
///     backend.create_field(horizontal_shape, 0.5_f32)?;
/// let mass_point_map_factor = backend.create_field(horizontal_shape, 2.0_f32)?;
/// let mass_multiplier = [1.0_f32; 4];
/// let offset = [0.0_f32; 4];
/// let region = MomentumCouplingRegion::try_new(
///     shape,
///     1..3,
///     1..3,
///     1..3,
///     1..4,
///     1..4,
///     1..4,
/// )?;
///
/// backend.couple_momentum(
///     MomentumCouplingOutputs::new(
///         &mut west_east_output,
///         &mut south_north_output,
///         &mut vertical_output,
///     ),
///     MomentumCouplingVelocities::new(
///         &west_east_velocity,
///         &south_north_velocity,
///         &vertical_velocity,
///     ),
///     MomentumCouplingMasses::new(
///         &west_east_mass,
///         &south_north_mass,
///         &mass_point_mass,
///     ),
///     MomentumCouplingMapFactors::new(
///         &west_east_map_factor,
///         &inverse_south_north_map_factor,
///         &mass_point_map_factor,
///     ),
///     MomentumCouplingCoefficients::new(
///         &mass_multiplier,
///         &offset,
///         &mass_multiplier,
///         &offset,
///     ),
///     &region,
/// )?;
/// assert_eq!(west_east_output.values()[21], 10.0);
/// assert_eq!(south_north_output.values()[21], 10.0);
/// assert_eq!(vertical_output.values()[21], 10.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait MomentumCouplingKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Produces mass-coupled west-east, south-north, and vertical momentum.
    ///
    /// # Errors
    ///
    /// Returns a typed error if a field shape or coefficient length differs
    /// from `region`, or if backend execution fails. All validation completes
    /// before any output changes.
    fn couple_momentum(
        &self,
        outputs: MomentumCouplingOutputs<'_, Self::Field>,
        velocities: MomentumCouplingVelocities<'_, Self::Field>,
        masses: MomentumCouplingMasses<'_, Self::Field>,
        map_factors: MomentumCouplingMapFactors<'_, Self::Field>,
        coefficients: MomentumCouplingCoefficients<'_>,
        region: &MomentumCouplingRegion,
    ) -> MomentumCouplingResult<()>;
}
