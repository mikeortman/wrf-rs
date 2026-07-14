use wrf_compute::FieldStorage;

use crate::{
    AcousticPressureCoefficients, AcousticPressureDampingPhase, AcousticPressureMasses,
    AcousticPressureMode, AcousticPressureParameters, AcousticPressureRegion,
    AcousticPressureResult, AcousticPressureState, AcousticPressureThermodynamics,
    AcousticPressureVerticalMetrics,
};

/// Backend capability for acoustic pressure and inverse-density diagnosis.
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
/// let mut inverse_density = backend.create_field(shape, 0.0)?;
/// let mut pressure = backend.create_field(shape, 0.0)?;
/// let mut geopotential = backend.create_field(shape, 900.0)?;
/// let mut previous_pressure = backend.create_field(shape, 0.0)?;
/// let full_inverse_density = backend.create_field(shape, 0.8)?;
/// let current_temperature = backend.create_field(shape, 2.0)?;
/// let reference_temperature = backend.create_field(shape, 1.0)?;
/// let pressure_coefficient = backend.create_field(shape, 140_000.0)?;
/// let perturbation_mass = backend.create_field(horizontal_shape, 1.0)?;
/// let full_mass = backend.create_field(horizontal_shape, 40.0)?;
/// let region = AcousticPressureRegion::try_new(
///     shape, 1..3, 1..3, 1..3, 1..4, 1..4, 1..4,
/// )?;
/// backend.update_acoustic_pressure(
///     AcousticPressureState::new(
///         &mut inverse_density, &mut pressure, &mut geopotential, &mut previous_pressure,
///     ),
///     AcousticPressureThermodynamics::new(
///         &full_inverse_density, &current_temperature, &reference_temperature,
///         &pressure_coefficient,
///     ),
///     AcousticPressureMasses::new(&perturbation_mass, &full_mass),
///     AcousticPressureCoefficients::new(&[0.2; 4], &[0.4; 4], &[1.1; 4]),
///     AcousticPressureVerticalMetrics::new(&[1.3; 4], &[0.7; 4]),
///     AcousticPressureParameters::new(300.0, 0.17),
///     AcousticPressureMode::Nonhydrostatic,
///     AcousticPressureDampingPhase::Initialize,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait AcousticPressureKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Updates pressure-system state in WRF `calc_p_rho` operation order.
    ///
    /// Every structural contract is validated before the first mutation. IEEE
    /// special values then pass through the exact governing-mode arithmetic.
    #[allow(clippy::too_many_arguments)]
    fn update_acoustic_pressure(
        &self,
        state: AcousticPressureState<'_, Self::Field>,
        thermodynamics: AcousticPressureThermodynamics<'_, Self::Field>,
        masses: AcousticPressureMasses<'_, Self::Field>,
        coefficients: AcousticPressureCoefficients<'_>,
        vertical_metrics: AcousticPressureVerticalMetrics<'_>,
        parameters: AcousticPressureParameters,
        mode: AcousticPressureMode,
        damping_phase: AcousticPressureDampingPhase,
        region: &AcousticPressureRegion,
    ) -> AcousticPressureResult<()>;
}
