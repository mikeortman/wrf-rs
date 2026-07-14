use wrf_compute::FieldStorage;

use crate::{
    VerticalAcousticCoefficientInputs, VerticalAcousticCoefficientParameters,
    VerticalAcousticCoefficientRegion, VerticalAcousticCoefficientResult,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics, VerticalAcousticSolveCoefficients,
};

/// Backend capability for WRF's vertically implicit acoustic coefficients.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::*;
///
/// let backend = CpuBackend::try_with_worker_count(4)?;
/// let shape = GridShape::try_new(4, 3, 5)?;
/// let mut lower = backend.create_field(shape, -999.0)?;
/// let mut inverse_diagonal = backend.create_field(shape, -999.0)?;
/// let mut upper = backend.create_field(shape, -999.0)?;
/// let mass = backend.create_field(shape.horizontal_shape(), 40.0)?;
/// let moisture = backend.create_field(shape, 1.0)?;
/// let pressure = backend.create_field(shape, 140_000.0)?;
/// let region = VerticalAcousticCoefficientRegion::try_new(
///     shape, 0..4, 0..3, 0..4, 0..4, 0..3,
/// )?;
/// backend.calculate_vertical_acoustic_coefficients(
///     VerticalAcousticSolveCoefficients::new(&mut lower, &mut inverse_diagonal, &mut upper),
///     VerticalAcousticCoefficientInputs::new(&mass, &moisture, &pressure),
///     VerticalAcousticMassCoefficients::new(
///         &[0.2; 5], &[0.4; 5], &[0.3; 5], &[0.5; 5],
///     ),
///     VerticalAcousticMetrics::new(&[1.2; 5], &[1.3; 5]),
///     VerticalAcousticCoefficientParameters::new(
///         2.0, 9.81, 0.1, VerticalAcousticTopBoundary::Nonrigid,
///     ),
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait VerticalAcousticCoefficientKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Constructs WRF `a`, `alpha`, and `gamma` in source operation order.
    ///
    /// Structural contracts are validated before the first output mutation.
    /// IEEE special values flow through the arithmetic without policy changes.
    fn calculate_vertical_acoustic_coefficients(
        &self,
        outputs: VerticalAcousticSolveCoefficients<'_, Self::Field>,
        inputs: VerticalAcousticCoefficientInputs<'_, Self::Field>,
        mass_coefficients: VerticalAcousticMassCoefficients<'_>,
        metrics: VerticalAcousticMetrics<'_>,
        parameters: VerticalAcousticCoefficientParameters,
        region: &VerticalAcousticCoefficientRegion,
    ) -> VerticalAcousticCoefficientResult<()>;
}
