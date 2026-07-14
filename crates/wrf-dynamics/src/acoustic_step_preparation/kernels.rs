use wrf_compute::FieldStorage;

use crate::{
    AcousticStepPreparationCoefficients, AcousticStepPreparationColumnMassTimeLevels,
    AcousticStepPreparationDiagnosticInputs, AcousticStepPreparationMapFactors,
    AcousticStepPreparationMassInputs, AcousticStepPreparationMassOutputs,
    AcousticStepPreparationPhase, AcousticStepPreparationRegion, AcousticStepPreparationResult,
    AcousticStepPreparationSavedOutputs, AcousticStepPreparationVolumeTimeLevels,
};

/// Backend capability for WRF acoustic small-step preparation.
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
/// let mut previous = [volume(1.0)?, volume(2.0)?, volume(-1.0)?, volume(300.0)?, volume(1_000.0)?];
/// let mut current = [volume(0.8)?, volume(1.6)?, volume(-0.7)?, volume(299.0)?, volume(900.0)?];
/// let mut saved = [volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?];
/// let mut previous_mass = horizontal(1.0)?;
/// let mut current_mass = horizontal(-0.5)?;
/// let mass_inputs = [horizontal(40.0)?, horizontal(42.0)?, horizontal(43.0)?, horizontal(44.0)?];
/// let mut mass_outputs = [horizontal(0.0)?, horizontal(0.0)?, horizontal(0.0)?, horizontal(0.0)?];
/// let diagnostics = [volume(80_000.0)?, volume(500.0)?, volume(0.8)?, volume(0.3)?];
/// let maps = [horizontal(1.03)?, horizontal(1.0 / 0.97)?, horizontal(1.12)?];
/// let mut saved_mass = horizontal(0.0)?;
/// let mut saved_omega = volume(0.0)?;
/// let mut pressure_coefficient = volume(0.0)?;
/// let region = AcousticStepPreparationRegion::try_new(
///     shape, 1..3, 1..3, 1..3, 1..4, 1..4, 1..4,
/// )?;
/// let [u1, v1, w1, t1, ph1] = &mut previous;
/// let [u2, v2, w2, t2, ph2] = &mut current;
/// let [us, vs, ws, ts, phs] = &mut saved;
/// let [muus, muvs, muts, mudf] = &mut mass_outputs;
/// backend.prepare_acoustic_step(
///     AcousticStepPreparationVolumeTimeLevels::new(u1, u2, v1, v2, w1, w2, t1, t2, ph1, ph2),
///     AcousticStepPreparationColumnMassTimeLevels::new(&mut previous_mass, &mut current_mass),
///     AcousticStepPreparationSavedOutputs::new(
///         us, vs, ws, ts, phs, &mut saved_mass, &mut saved_omega, &mut pressure_coefficient,
///     ),
///     AcousticStepPreparationMassOutputs::new(muus, muvs, muts, mudf),
///     AcousticStepPreparationMassInputs::new(
///         &mass_inputs[0], &mass_inputs[1], &mass_inputs[2], &mass_inputs[3],
///     ),
///     AcousticStepPreparationDiagnosticInputs::new(
///         &diagnostics[0], &diagnostics[1], &diagnostics[2], &diagnostics[3],
///     ),
///     AcousticStepPreparationMapFactors::new(&maps[0], &maps[1], &maps[2]),
///     AcousticStepPreparationCoefficients::new(&[0.2; 4], &[0.4; 4], &[0.3; 4], &[0.5; 4]),
///     AcousticStepPreparationPhase::FirstSubstep,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait AcousticStepPreparationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Switches time levels and forms the coupled perturbation variables.
    ///
    /// Every field, coefficient, range, and later-substep neighbor is validated
    /// before the first mutation. IEEE values then follow WRF arithmetic.
    #[allow(clippy::too_many_arguments)]
    fn prepare_acoustic_step(
        &self,
        volume_time_levels: AcousticStepPreparationVolumeTimeLevels<'_, Self::Field>,
        column_mass_time_levels: AcousticStepPreparationColumnMassTimeLevels<'_, Self::Field>,
        saved_outputs: AcousticStepPreparationSavedOutputs<'_, Self::Field>,
        mass_outputs: AcousticStepPreparationMassOutputs<'_, Self::Field>,
        mass_inputs: AcousticStepPreparationMassInputs<'_, Self::Field>,
        diagnostics: AcousticStepPreparationDiagnosticInputs<'_, Self::Field>,
        map_factors: AcousticStepPreparationMapFactors<'_, Self::Field>,
        coefficients: AcousticStepPreparationCoefficients<'_>,
        phase: AcousticStepPreparationPhase,
        region: &AcousticStepPreparationRegion,
    ) -> AcousticStepPreparationResult<()>;
}
