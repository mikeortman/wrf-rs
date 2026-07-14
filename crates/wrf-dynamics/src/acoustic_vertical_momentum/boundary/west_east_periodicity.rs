/// Whether specified or nested west-east edges remain active through wrapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticVerticalWestEastPeriodicity {
    /// West-east edges are ordinary nonperiodic boundaries.
    Nonperiodic,
    /// West-east edges wrap and remain active.
    Periodic,
}

impl AcousticVerticalWestEastPeriodicity {
    pub(crate) const fn is_periodic(self) -> bool {
        matches!(self, Self::Periodic)
    }
}
