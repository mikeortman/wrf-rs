use wrf_compute::FieldStorage;

use crate::{
    DryTendencyAssemblyCoefficients, DryTendencyAssemblyForwardTendencies,
    DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase, DryTendencyAssemblyRegion,
    DryTendencyAssemblyResult, DryTendencyAssemblyRungeKuttaTendencies,
    DryTendencyAssemblySavedTendencies, DryTendencyAssemblyThermodynamics,
};

/// Backend capability for ARW dry large-timestep tendency assembly.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::*;
///
/// let backend = CpuBackend::try_with_worker_count(4)?;
/// let shape = GridShape::try_new(4, 4, 4)?;
/// let horizontal_shape = shape.horizontal_shape();
/// let volume = |value| backend.create_field(shape, value);
/// let horizontal = |value| backend.create_field(horizontal_shape, value);
/// let mut rk = [volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?];
/// let mut forward = [volume(0.1)?, volume(0.1)?, volume(0.1)?, volume(0.1)?, volume(0.1)?];
/// let saved = [volume(0.01)?, volume(0.01)?, volume(0.01)?, volume(0.01)?, volume(0.01)?];
/// let heating = volume(0.001)?;
/// let mut rk_mass = horizontal(0.0)?;
/// let forward_mass = horizontal(0.1)?;
/// let full_mass = horizontal(100.0)?;
/// let maps = [horizontal(1.0)?, horizontal(1.0)?, horizontal(1.0)?, horizontal(1.0)?];
/// let region = DryTendencyAssemblyRegion::try_new(
///     shape, 1..3, 1..3, 1..3, 1..4, 1..4, 1..4,
/// )?;
/// let [ru, rv, rw, ph, t] = &mut rk;
/// let [ruf, rvf, rwf, phf, tf] = &mut forward;
/// backend.assemble_dry_tendencies(
///     DryTendencyAssemblyRungeKuttaTendencies::new(ru, rv, rw, ph, t, &mut rk_mass),
///     DryTendencyAssemblyForwardTendencies::new(ruf, rvf, rwf, phf, tf, &forward_mass),
///     DryTendencyAssemblySavedTendencies::new(&saved[0], &saved[1], &saved[2], &saved[3], &saved[4]),
///     DryTendencyAssemblyThermodynamics::new(&heating, &full_mass),
///     DryTendencyAssemblyMapFactors::new(&maps[0], &maps[1], &maps[2], &maps[3]),
///     DryTendencyAssemblyCoefficients::new(&[0.2; 4], &[0.4; 4]),
///     DryTendencyAssemblyPhase::FirstSubstep,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait DryTendencyAssemblyKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Combines persistent and substep tendencies in WRF source order.
    ///
    /// All structural contracts are validated before any output changes. IEEE
    /// non-finite values are deliberately passed through WRF's arithmetic.
    #[allow(clippy::too_many_arguments)]
    fn assemble_dry_tendencies(
        &self,
        runge_kutta: DryTendencyAssemblyRungeKuttaTendencies<'_, Self::Field>,
        forward: DryTendencyAssemblyForwardTendencies<'_, Self::Field>,
        saved: DryTendencyAssemblySavedTendencies<'_, Self::Field>,
        thermodynamics: DryTendencyAssemblyThermodynamics<'_, Self::Field>,
        map_factors: DryTendencyAssemblyMapFactors<'_, Self::Field>,
        coefficients: DryTendencyAssemblyCoefficients<'_>,
        phase: DryTendencyAssemblyPhase,
        region: &DryTendencyAssemblyRegion,
    ) -> DryTendencyAssemblyResult<()>;
}
