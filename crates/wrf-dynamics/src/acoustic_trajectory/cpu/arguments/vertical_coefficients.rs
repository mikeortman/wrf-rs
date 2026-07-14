use wrf_compute::CpuField;

use crate::{
    VerticalAcousticCoefficientInputs, VerticalAcousticCoefficientParameters,
    VerticalAcousticCoefficientRegion, VerticalAcousticMassCoefficients, VerticalAcousticMetrics,
    VerticalAcousticSolveCoefficients,
};

pub(crate) struct VerticalCoefficientArguments<'a, 'coefficients, 'region> {
    pub(crate) outputs: VerticalAcousticSolveCoefficients<'a, CpuField<f32>>,
    pub(crate) inputs: VerticalAcousticCoefficientInputs<'a, CpuField<f32>>,
    pub(crate) mass_coefficients: VerticalAcousticMassCoefficients<'coefficients>,
    pub(crate) metrics: VerticalAcousticMetrics<'coefficients>,
    pub(crate) parameters: VerticalAcousticCoefficientParameters,
    pub(crate) region: &'region VerticalAcousticCoefficientRegion,
}
