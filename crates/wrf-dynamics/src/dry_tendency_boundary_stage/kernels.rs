use wrf_compute::FieldStorage;

use super::{
    DryTendencyBoundaryStageControls, DryTendencyBoundaryStageInputs,
    DryTendencyBoundaryStageRegions, DryTendencyBoundaryStageResult,
    DryTendencyBoundaryStageVertical,
};
use crate::DryTendencyAssemblyRungeKuttaTendencies;

/// Backend capability for WRF's coupled dry-tendency boundary stage.
///
/// Implementations may retain backend-native storage. The CPU implementation
/// preflights both source routines before mutation and then executes them in
/// pinned WRF v4.7.1 order.
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
/// let boundary_shape = GridShape::try_new(4, 1, 3)?;
/// let boundary = [
///     backend.create_field(boundary_shape, 0.2)?,
///     backend.create_field(boundary_shape, 0.2)?,
///     backend.create_field(boundary_shape, 0.2)?,
///     backend.create_field(boundary_shape, 0.2)?,
/// ];
/// let mass_boundary_shape = GridShape::try_new(4, 1, 1)?;
/// let mass_boundary = [
///     backend.create_field(mass_boundary_shape, 0.2)?,
///     backend.create_field(mass_boundary_shape, 0.2)?,
///     backend.create_field(mass_boundary_shape, 0.2)?,
///     backend.create_field(mass_boundary_shape, 0.2)?,
/// ];
/// let volume_boundaries = SpecifiedBoundaryTendencies::new(
///     &boundary[0], &boundary[1], &boundary[2], &boundary[3],
/// );
/// let mass_boundaries = SpecifiedBoundaryTendencies::new(
///     &mass_boundary[0], &mass_boundary[1], &mass_boundary[2], &mass_boundary[3],
/// );
/// let regions = DryTendencyBoundaryStageRegions::try_new(
///     shape, 1..3, 1..3, 1..3, 1..4, 1..4, 1..4,
/// )?;
/// let [u, v, w, ph, t] = &mut rk;
/// let [uf, vf, wf, phf, tf] = &mut forward;
/// backend.advance_dry_tendency_boundary_stage(
///     DryTendencyAssemblyRungeKuttaTendencies::new(u, v, w, ph, t, &mut rk_mass),
///     DryTendencyBoundaryStageInputs::new(
///         DryTendencyAssemblyForwardTendencies::new(uf, vf, wf, phf, tf, &forward_mass),
///         DryTendencyAssemblySavedTendencies::new(
///             &saved[0], &saved[1], &saved[2], &saved[3], &saved[4],
///         ),
///         DryTendencyAssemblyThermodynamics::new(&heating, &full_mass),
///         DryTendencyAssemblyMapFactors::new(&maps[0], &maps[1], &maps[2], &maps[3]),
///         DryTendencyAssemblyCoefficients::new(&[0.2; 4], &[0.4; 4]),
///         DryBoundaryTendencyBoundaryFields::new(
///             volume_boundaries, volume_boundaries, volume_boundaries,
///             volume_boundaries, mass_boundaries,
///         ),
///     ),
///     DryTendencyBoundaryStageVertical::Global,
///     DryTendencyBoundaryStageControls::new(
///         DryTendencyAssemblyPhase::FirstSubstep,
///         SpecifiedBoundaryTendencyParameters::new(1, 1),
///         SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     ),
///     &regions,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait DryTendencyBoundaryStageKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Assembles dry tendencies and then assigns specified boundary tendencies.
    ///
    /// # Errors
    ///
    /// Returns a typed assembly or boundary-assignment failure. Structural
    /// validation is atomic across the complete coupled operation.
    fn advance_dry_tendency_boundary_stage(
        &self,
        runge_kutta: DryTendencyAssemblyRungeKuttaTendencies<'_, Self::Field>,
        inputs: DryTendencyBoundaryStageInputs<'_, Self::Field>,
        vertical: DryTendencyBoundaryStageVertical<'_, Self::Field>,
        controls: DryTendencyBoundaryStageControls,
        regions: &DryTendencyBoundaryStageRegions,
    ) -> DryTendencyBoundaryStageResult<()>;
}
