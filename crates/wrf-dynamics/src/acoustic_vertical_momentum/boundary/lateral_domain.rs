/// Lateral-domain treatment used to select active mass points.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticVerticalLateralDomain {
    /// A global or otherwise unrestricted domain.
    Global,
    /// A specified-boundary or nested domain that excludes its outer row.
    SpecifiedOrNested,
}

impl AcousticVerticalLateralDomain {
    pub(crate) const fn excludes_edge_points(self) -> bool {
        matches!(self, Self::SpecifiedOrNested)
    }
}
