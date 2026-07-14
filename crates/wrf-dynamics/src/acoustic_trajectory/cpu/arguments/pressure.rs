use wrf_compute::CpuField;

use crate::{
    AcousticPressureCoefficients, AcousticPressureDampingPhase, AcousticPressureMasses,
    AcousticPressureMode, AcousticPressureParameters, AcousticPressureRegion,
    AcousticPressureState, AcousticPressureThermodynamics, AcousticPressureVerticalMetrics,
};

pub(crate) struct PressureArguments<'a, 'coefficients, 'region> {
    pub(crate) state: AcousticPressureState<'a, CpuField<f32>>,
    pub(crate) thermodynamics: AcousticPressureThermodynamics<'a, CpuField<f32>>,
    pub(crate) masses: AcousticPressureMasses<'a, CpuField<f32>>,
    pub(crate) coefficients: AcousticPressureCoefficients<'coefficients>,
    pub(crate) vertical_metrics: AcousticPressureVerticalMetrics<'coefficients>,
    pub(crate) parameters: AcousticPressureParameters,
    pub(crate) mode: AcousticPressureMode,
    pub(crate) damping_phase: AcousticPressureDampingPhase,
    pub(crate) region: &'region AcousticPressureRegion,
}
