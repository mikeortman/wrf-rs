/// Physical U-momentum boundary behavior at one west-east edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticWestEastBoundary {
    /// Neither open nor symmetric treatment is active.
    Closed,
    /// Pressure-gradient advancement excludes the boundary momentum point.
    Open,
    /// Tendency and pressure-gradient advancement both exclude the boundary point.
    Symmetric,
}

impl AcousticWestEastBoundary {
    pub(crate) const fn excludes_pressure_gradient(self) -> bool {
        matches!(self, Self::Open | Self::Symmetric)
    }

    pub(crate) const fn excludes_tendency(self) -> bool {
        matches!(self, Self::Symmetric)
    }
}
