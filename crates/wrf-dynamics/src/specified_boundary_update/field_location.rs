/// C-grid and vertical location of a field receiving specified tendencies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryFieldLocation {
    /// Mass points on half levels, such as perturbation potential temperature.
    MassHalfLevel,
    /// West–east faces on half levels, WRF selector `u`.
    WestEastFace,
    /// South–north faces on half levels, WRF selector `v`.
    SouthNorthFace,
    /// A horizontal mass field represented by one stored vertical level, WRF `m`.
    HorizontalMass,
    /// Mass points including the upper full level, WRF selector `h`.
    FullLevel,
}

impl SpecifiedBoundaryFieldLocation {
    pub(crate) const fn has_upper_west_east_point(self) -> bool {
        matches!(self, Self::WestEastFace)
    }

    pub(crate) const fn has_upper_south_north_point(self) -> bool {
        matches!(self, Self::SouthNorthFace)
    }

    pub(crate) const fn has_upper_vertical_point(self) -> bool {
        matches!(self, Self::FullLevel)
    }
}
