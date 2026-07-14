/// Scalar parameters for pressure diagnosis and divergence damping.
#[derive(Clone, Copy, Debug)]
pub struct AcousticPressureParameters {
    pub(crate) reference_temperature: f32,
    pub(crate) divergence_damping: f32,
}

impl AcousticPressureParameters {
    /// Preserves WRF's `t0` and `smdiv` values, including IEEE special values.
    pub const fn new(reference_temperature: f32, divergence_damping: f32) -> Self {
        Self {
            reference_temperature,
            divergence_damping,
        }
    }
}
