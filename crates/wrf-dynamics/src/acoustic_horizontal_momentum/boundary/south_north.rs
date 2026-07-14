/// Physical V-momentum boundary behavior at one south-north edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticSouthNorthBoundary {
    /// Neither open, symmetric, nor polar treatment is active.
    Closed,
    /// Pressure-gradient advancement excludes the boundary momentum point.
    Open,
    /// Tendency and pressure-gradient advancement both exclude the boundary point.
    Symmetric,
    /// Pressure-gradient advancement is suppressed and V is forced to zero.
    Polar,
}

impl AcousticSouthNorthBoundary {
    pub(crate) const fn excludes_pressure_gradient(self) -> bool {
        matches!(self, Self::Open | Self::Symmetric | Self::Polar)
    }

    pub(crate) const fn excludes_tendency(self) -> bool {
        matches!(self, Self::Symmetric)
    }

    pub(crate) const fn is_polar(self) -> bool {
        matches!(self, Self::Polar)
    }
}
