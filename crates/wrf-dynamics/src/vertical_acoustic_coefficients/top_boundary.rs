/// Upper-boundary policy for the vertically implicit acoustic solve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalAcousticTopBoundary {
    /// WRF `top_lid = .false.`; retain the upper boundary coupling.
    Nonrigid,
    /// WRF `top_lid = .true.`; multiply the upper lower-diagonal term by zero.
    RigidLid,
}

impl VerticalAcousticTopBoundary {
    pub(crate) const fn lower_diagonal_multiplier(self) -> f32 {
        match self {
            Self::Nonrigid => 1.0,
            Self::RigidLid => 0.0,
        }
    }
}
