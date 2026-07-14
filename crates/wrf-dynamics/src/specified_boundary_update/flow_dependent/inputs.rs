use wrf_compute::FieldStorage;

/// Borrowed coupled velocities that classify inflow and outflow.
#[derive(Clone, Copy)]
pub struct SpecifiedBoundaryFlowInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_velocity: &'a Field,
    pub(crate) south_north_velocity: &'a Field,
}

impl<'a, Field> SpecifiedBoundaryFlowInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the WRF U and V velocity fields without copying their storage.
    pub const fn new(west_east_velocity: &'a Field, south_north_velocity: &'a Field) -> Self {
        Self {
            west_east_velocity,
            south_north_velocity,
        }
    }
}
