/// Whether west/east specified-zone updates are suppressed by periodic X.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryWestEastPeriodicity {
    /// Apply all contacted specified boundaries.
    Nonperiodic,
    /// Suppress west/east updates and retain rectangular south/north zones.
    Periodic,
}

impl SpecifiedBoundaryWestEastPeriodicity {
    pub(crate) const fn is_periodic(self) -> bool {
        matches!(self, Self::Periodic)
    }
}
