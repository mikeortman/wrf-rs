use wrf_compute::FieldStorage;

/// Reusable storage for the geopotential right-hand side.
#[derive(Debug)]
pub struct AcousticVerticalWorkspace<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) geopotential_right_hand_side: &'a mut Field,
}

impl<'a, Field> AcousticVerticalWorkspace<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Borrows one volume field allocated during model setup.
    pub const fn new(geopotential_right_hand_side: &'a mut Field) -> Self {
        Self {
            geopotential_right_hand_side,
        }
    }
}
