use wrf_compute::FieldStorage;

/// Perturbation and full column masses used by pressure diagnosis.
#[derive(Clone, Copy, Debug)]
pub struct AcousticPressureMasses<'a, Field: FieldStorage<f32>> {
    pub(crate) perturbation: &'a Field,
    pub(crate) full: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticPressureMasses<'a, Field> {
    /// Groups WRF `mu` and `mut` without copying them.
    pub const fn new(perturbation: &'a Field, full: &'a Field) -> Self {
        Self { perturbation, full }
    }
}
