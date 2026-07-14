use wrf_compute::FieldStorage;

use crate::{
    ColumnMassStaggeringPeriodicity, RungeKuttaPreparationInputs, RungeKuttaPreparationOutputs,
    RungeKuttaPreparationRegions, RungeKuttaPreparationResult,
};

/// Backend capability for the diagnostic preparation before an ARW RK step.
///
/// Implementations reproduce the seven calls in WRF v4.7.1 `rk_step_prep` in
/// their original order. The API retains native field storage, so a GPU backend
/// may fuse stages without round-tripping through host memory. The CPU backend
/// composes the already parity-tested kernels on its persistent worker pool.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::*;
///
/// let backend = CpuBackend::try_with_worker_count(4)?;
/// let shape = GridShape::try_new(6, 6, 5)?;
/// let horizontal_shape = shape.horizontal_shape();
/// let volume = |value| backend.create_field(shape, value);
/// let horizontal = |value| backend.create_field(horizontal_shape, value);
/// let perturbation_mass = horizontal(10.0)?;
/// let base_mass = horizontal(90.0)?;
/// let velocities = [volume(1.0)?, volume(-0.5)?, volume(0.25)?];
/// let map_factors = [horizontal(1.0)?, horizontal(1.1)?, horizontal(0.9)?, horizontal(0.8)?];
/// let moisture = vec![volume(0.001)?, volume(0.002)?];
/// let thermodynamics = [volume(0.2)?, volume(0.8)?, volume(100.0)?, volume(1_000.0)?];
/// let mut mass_outputs = [horizontal(-1.0)?, horizontal(-1.0)?, horizontal(-1.0)?];
/// let mut momentum_outputs = [volume(-1.0)?, volume(-1.0)?, volume(-1.0)?];
/// let mut diagnostics = [
///     volume(-1.0)?, volume(-1.0)?, volume(-1.0)?,
///     volume(-1.0)?, volume(-1.0)?, volume(-1.0)?,
/// ];
/// let inputs = RungeKuttaPreparationInputs::new(
///     RungeKuttaPreparationMassInputs::new(&perturbation_mass, &base_mass),
///     RungeKuttaPreparationVelocities::new(&velocities[0], &velocities[1], &velocities[2]),
///     RungeKuttaPreparationMapFactors::new(
///         &map_factors[0], &map_factors[1], &map_factors[2], &map_factors[3],
///     ),
///     RungeKuttaPreparationCoefficients::new(
///         &[1.0; 5], &[0.1; 5], &[0.9; 5], &[0.2; 5], &[-0.25; 5],
///     ),
///     MoistureSpecies::new(&moisture),
///     RungeKuttaPreparationThermodynamicInputs::new(
///         &thermodynamics[0], &thermodynamics[1], &thermodynamics[2], &thermodynamics[3],
///     ),
///     OmegaDiagnosisGridMetrics::new(1.0, 1.0),
/// );
/// let [full_mass, west_east_mass, south_north_mass] = &mut mass_outputs;
/// let [west_east_momentum, south_north_momentum, vertical_momentum] = &mut momentum_outputs;
/// let [omega, cqu, cqv, cqw, inverse_density, geopotential] = &mut diagnostics;
/// let outputs = RungeKuttaPreparationOutputs::new(
///     RungeKuttaPreparationMassOutputs::new(full_mass, west_east_mass, south_north_mass),
///     RungeKuttaPreparationMomentumOutputs::new(
///         west_east_momentum, south_north_momentum, vertical_momentum,
///     ),
///     RungeKuttaPreparationDiagnosticOutputs::new(
///         omega, cqu, cqv, cqw, inverse_density, geopotential,
///     ),
/// );
/// let regions = RungeKuttaPreparationRegions::new(
///     ColumnMassStaggeringRegion::try_new(horizontal_shape, 1..5, 1..5, 2..5, 2..5)?,
///     MomentumCouplingRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5)?,
///     OmegaDiagnosisRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5)?,
///     MoistureCoefficientRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..5)?,
///     InverseDensityRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..4)?,
///     PressurePointGeopotentialRegion::try_new(shape, 1..5, 1..5, 1..4, 2..5, 2..5, 1..4)?,
/// );
/// backend.prepare_runge_kutta_step(
///     outputs, inputs, &regions, ColumnMassStaggeringPeriodicity::None,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait RungeKuttaPreparationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Produces all column-mass, momentum, omega, moisture, and state diagnostics.
    ///
    /// Every field, coefficient, neighbor, and halo contract is checked before
    /// the first output changes. Validation failures therefore leave all twelve
    /// outputs untouched. Execution failures retain the component kernel's
    /// stage-specific error.
    fn prepare_runge_kutta_step(
        &self,
        outputs: RungeKuttaPreparationOutputs<'_, Self::Field>,
        inputs: RungeKuttaPreparationInputs<'_, Self::Field>,
        regions: &RungeKuttaPreparationRegions,
        periodicity: ColumnMassStaggeringPeriodicity,
    ) -> RungeKuttaPreparationResult<()>;
}
