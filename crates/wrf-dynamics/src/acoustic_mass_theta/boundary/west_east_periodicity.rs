/// West-east periodicity used by the acoustic mass-and-theta tile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticMassThetaWestEastPeriodicity {
    /// Apply any specified/nested west-east edge exclusion.
    Nonperiodic,
    /// Retain the ordinary west-east tile range.
    Periodic,
}

impl AcousticMassThetaWestEastPeriodicity {
    pub(crate) const fn is_periodic(self) -> bool {
        matches!(self, Self::Periodic)
    }
}
