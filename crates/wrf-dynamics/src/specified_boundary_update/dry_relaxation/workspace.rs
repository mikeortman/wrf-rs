use wrf_compute::FieldStorage;

/// Caller-owned tile-halo workspace reused for mass-weighted fields.
pub struct DryBoundaryRelaxationWorkspace<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) mass_weighted_field: &'a mut Field,
}

impl<'a, Field> DryBoundaryRelaxationWorkspace<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Borrows reusable backend-native storage without allocating in the kernel.
    pub const fn new(mass_weighted_field: &'a mut Field) -> Self {
        Self {
            mass_weighted_field,
        }
    }
}
