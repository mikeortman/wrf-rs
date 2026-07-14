use wrf_compute::FieldStorage;

/// Reusable caller-owned scratch for the complete acoustic trajectory.
#[derive(Debug)]
pub struct AcousticTrajectoryWorkspace<'a, Field: FieldStorage<f32>> {
    pub(crate) geopotential_right_hand_side: &'a mut Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryWorkspace<'a, Field> {
    /// Borrows the one volume workspace required by the vertical solve.
    pub const fn new(geopotential_right_hand_side: &'a mut Field) -> Self {
        Self {
            geopotential_right_hand_side,
        }
    }
}
