/// West-east periodicity used while deriving acoustic relaxation-zone bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticWestEastPeriodicity {
    /// Use nonperiodic west-east clipping.
    Nonperiodic,
    /// Override relaxation-zone clipping with periodic west-east tile bounds.
    Periodic,
}

impl AcousticWestEastPeriodicity {
    pub(crate) const fn is_periodic(self) -> bool {
        matches!(self, Self::Periodic)
    }
}
