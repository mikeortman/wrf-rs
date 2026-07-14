use wrf_compute::FieldStorage;

use super::{
    DryLargeStepBoundaryStageControls, DryLargeStepBoundaryStageInputs,
    DryLargeStepBoundaryStageMode, DryLargeStepBoundaryStageRegions,
    DryLargeStepBoundaryStageResult, DryLargeStepSavedTendencies,
};
use crate::DryTendencyAssemblyRungeKuttaTendencies;

/// Backend capability for WRF's complete dry large-step boundary stage.
///
/// Implementations may retain backend-native storage. The CPU implementation
/// preflights every active source routine before the first mutation and then
/// executes `relax_bdy_dry`, `rk_addtend_dry`, and `spec_bdy_dry` in pinned
/// WRF v4.7.1 order.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::*;
///
/// let backend = CpuBackend::try_with_worker_count(4)?;
/// let shape = GridShape::try_new(8, 8, 6)?;
/// let horizontal_shape = shape.horizontal_shape();
/// let regions = DryLargeStepBoundaryStageRegions::try_new(
///     shape, 1..7, 1..7, 1..5, 1..7, 1..7, 1..6,
/// )?;
/// let volume = |value| backend.create_field(shape, value);
/// let horizontal = |value| backend.create_field(horizontal_shape, value);
/// let mut rk = [volume(1.0)?, volume(2.0)?, volume(-1.0)?, volume(3.0)?, volume(-2.0)?];
/// let mut forward = [volume(0.3)?, volume(-0.4)?, volume(0.5)?, volume(-0.6)?, volume(0.7)?];
/// let mut saved = [volume(0.09)?, volume(-0.08)?, volume(0.07)?, volume(-0.06)?, volume(0.05)?];
/// let state = [volume(10.0)?, volume(20.0)?, volume(30.0)?, volume(40.0)?, volume(50.0)?];
/// let heating = volume(0.001)?;
/// let mut rk_mass = horizontal(0.6)?;
/// let forward_mass = horizontal(-0.2)?;
/// let perturbation_mass = horizontal(60.0)?;
/// let full_mass = horizontal(50.0)?;
/// let maps = [horizontal(1.03)?, horizontal(0.97)?, horizontal(1.0 / 0.97)?, horizontal(1.12)?];
/// let volume_boundary_shape = GridShape::try_new(8, 5, 5)?;
/// let horizontal_boundary_shape = GridShape::try_new(8, 5, 1)?;
/// let volume_values = backend.create_field(volume_boundary_shape, 2.0)?;
/// let volume_slabs = backend.create_field(volume_boundary_shape, 0.1)?;
/// let horizontal_values = backend.create_field(horizontal_boundary_shape, 2.0)?;
/// let horizontal_slabs = backend.create_field(horizontal_boundary_shape, 0.1)?;
/// let values = SpecifiedBoundaryRelaxationBoundaryValues::new(
///     &volume_values, &volume_values, &volume_values, &volume_values,
/// );
/// let slabs = SpecifiedBoundaryTendencies::new(
///     &volume_slabs, &volume_slabs, &volume_slabs, &volume_slabs,
/// );
/// let mass_values = SpecifiedBoundaryRelaxationBoundaryValues::new(
///     &horizontal_values, &horizontal_values, &horizontal_values, &horizontal_values,
/// );
/// let mass_slabs = SpecifiedBoundaryTendencies::new(
///     &horizontal_slabs, &horizontal_slabs, &horizontal_slabs, &horizontal_slabs,
/// );
/// let mut workspace = backend.create_field(regions.relaxation_workspace_shape(), 0.0)?;
/// let [ru, rv, rw, ph, t] = &mut rk;
/// let [ruf, rvf, rwf, phf, tf] = &mut forward;
/// let [us, vs, ws, phs, ts] = &mut saved;
/// backend.advance_dry_large_step_boundary_stage(
///     DryTendencyAssemblyRungeKuttaTendencies::new(ru, rv, rw, ph, t, &mut rk_mass),
///     DryLargeStepSavedTendencies::new(us, vs, ws, phs, ts),
///     DryLargeStepBoundaryStageInputs::new(
///         DryTendencyAssemblyForwardTendencies::new(ruf, rvf, rwf, phf, tf, &forward_mass),
///         DryTendencyAssemblyThermodynamics::new(&heating, &full_mass),
///         DryTendencyAssemblyMapFactors::new(&maps[0], &maps[1], &maps[2], &maps[3]),
///         DryTendencyAssemblyCoefficients::new(&[0.2; 6], &[0.4; 6]),
///         DryBoundaryTendencyBoundaryFields::new(slabs, slabs, slabs, slabs, mass_slabs),
///     ),
///     DryLargeStepBoundaryStageMode::FirstSubstepNested {
///         relaxation: DryLargeStepRelaxationInputs::new(
///             DryBoundaryRelaxationState::new(
///                 &state[0], &state[1], &state[2], &state[3],
///                 &perturbation_mass, &full_mass,
///             ),
///             DryLargeStepRelaxationBoundaryValues::new(
///                 values, values, values, values, mass_values,
///             ),
///             DryBoundaryRelaxationWorkspace::new(&mut workspace),
///             DryBoundaryRelaxationMassCoefficients::new(
///                 &[0.6; 6], &[0.4; 6], &[0.55; 6], &[0.45; 6],
///             ),
///             SpecifiedBoundaryRelaxationCoefficients::new(
///                 &[0.0, 0.7, 0.5, 0.3, 0.0], &[0.0, 0.1, 0.08, 0.04, 0.0],
///             ),
///             SpecifiedBoundaryRelaxationParameters::new(5, 1, 4, 0.25),
///         ),
///         vertical: DryLargeStepNestedVertical::new(&state[4], values, slabs),
///     },
///     DryLargeStepBoundaryStageControls::new(
///         SpecifiedBoundaryTendencyParameters::new(5, 1),
///         SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     ),
///     &regions,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait DryLargeStepBoundaryStageKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Relaxes, assembles, and assigns dry tendencies in WRF source order.
    ///
    /// The saved tendencies are owned mutably once: first-substep relaxation
    /// receives mutable reborrows and assembly then reads them immutably. The
    /// Runge–Kutta column-mass tendency (`mu_tend`) flows mutably through all
    /// three routines.
    ///
    /// # Errors
    ///
    /// Returns a typed relaxation, assembly, or boundary-assignment failure.
    /// Structural validation is atomic across the complete composed stage.
    fn advance_dry_large_step_boundary_stage(
        &self,
        runge_kutta: DryTendencyAssemblyRungeKuttaTendencies<'_, Self::Field>,
        saved: DryLargeStepSavedTendencies<'_, Self::Field>,
        inputs: DryLargeStepBoundaryStageInputs<'_, Self::Field>,
        mode: DryLargeStepBoundaryStageMode<'_, Self::Field>,
        controls: DryLargeStepBoundaryStageControls,
        regions: &DryLargeStepBoundaryStageRegions,
    ) -> DryLargeStepBoundaryStageResult<()>;
}
