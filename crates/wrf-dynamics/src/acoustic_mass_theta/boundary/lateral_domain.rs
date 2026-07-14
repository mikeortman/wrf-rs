/// Lateral-domain treatment used to derive active mass-point ranges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticMassThetaLateralDomain {
    /// Ordinary global-domain bounds.
    Global,
    /// WRF `specified .or. nested`; exclude one mass point at each edge.
    SpecifiedOrNested,
}

impl AcousticMassThetaLateralDomain {
    pub(crate) const fn excludes_edge_points(self) -> bool {
        matches!(self, Self::SpecifiedOrNested)
    }
}
