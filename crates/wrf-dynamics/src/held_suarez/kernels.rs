use wrf_compute::FieldStorage;

use crate::{HeldSuarezDampingFields, HeldSuarezDampingRegion, HeldSuarezDampingResult};

/// Backend capability for Held-Suarez idealized momentum damping.
///
/// The capability keeps native field storage behind the backend while exposing
/// the same validated scientific operation to CPU and future GPU callers.
/// Finite inputs retain WRF's single-precision ordering. Non-finite inputs are
/// not rejected with an extra domain-sized scan; NaN results have class parity
/// with the pinned Fortran oracle but do not promise a payload bit pattern.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     HeldSuarezDampingFields, HeldSuarezDampingKernels,
///     HeldSuarezDampingRegion,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(3, 3, 2)?;
/// let mut west_east_tendency = backend.create_field(shape, 0.0_f32)?;
/// let mut south_north_tendency = backend.create_field(shape, 0.0_f32)?;
/// let west_east_momentum = backend.create_field(shape, 10.0_f32)?;
/// let south_north_momentum = backend.create_field(shape, -5.0_f32)?;
/// let perturbation_pressure = backend.create_field(shape, 0.0_f32)?;
/// let base_pressure = backend.create_field(shape, 100_000.0_f32)?;
/// let region = HeldSuarezDampingRegion::try_new(
///     shape,
///     1..3,
///     1..2,
///     1..3,
///     1..3,
///     1,
/// )?;
///
/// backend.apply_held_suarez_damping(
///     HeldSuarezDampingFields::new(
///         &mut west_east_tendency,
///         &mut south_north_tendency,
///         &west_east_momentum,
///         &south_north_momentum,
///         &perturbation_pressure,
///         &base_pressure,
///     ),
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait HeldSuarezDampingKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Applies pressure-dependent Rayleigh damping to momentum tendencies.
    ///
    /// # Errors
    ///
    /// Returns an error if any field shape differs from the region or CPU
    /// execution fails.
    fn apply_held_suarez_damping(
        &self,
        fields: HeldSuarezDampingFields<'_, Self::Field>,
        region: &HeldSuarezDampingRegion,
    ) -> HeldSuarezDampingResult<()>;
}
