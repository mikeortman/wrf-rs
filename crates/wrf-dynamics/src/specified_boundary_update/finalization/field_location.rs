use crate::SpecifiedBoundaryFieldLocation;

/// Field location and normalization policy used by WRF `spec_bdy_final`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryFinalizationFieldLocation {
    /// Unstaggered half-level scalar normalized by column mass.
    MassHalfLevel,
    /// West–east momentum normalized by column mass and its map factor.
    WestEastMomentum,
    /// South–north momentum normalized by column mass and its map factor.
    SouthNorthMomentum,
    /// Vertical momentum on full levels, normalized by column mass and map factor.
    VerticalMomentum,
    /// Two-dimensional column mass without mass or map-factor normalization.
    HorizontalMass,
    /// Unstaggered full-level field normalized by column mass.
    FullLevel,
}

impl SpecifiedBoundaryFinalizationFieldLocation {
    pub(crate) const fn geometry_location(self) -> SpecifiedBoundaryFieldLocation {
        match self {
            Self::MassHalfLevel => SpecifiedBoundaryFieldLocation::MassHalfLevel,
            Self::WestEastMomentum => SpecifiedBoundaryFieldLocation::WestEastFace,
            Self::SouthNorthMomentum => SpecifiedBoundaryFieldLocation::SouthNorthFace,
            Self::VerticalMomentum | Self::FullLevel => SpecifiedBoundaryFieldLocation::FullLevel,
            Self::HorizontalMass => SpecifiedBoundaryFieldLocation::HorizontalMass,
        }
    }

    pub(crate) const fn uses_column_mass(self) -> bool {
        !matches!(self, Self::HorizontalMass)
    }

    pub(crate) const fn uses_map_factor(self) -> bool {
        matches!(
            self,
            Self::WestEastMomentum | Self::SouthNorthMomentum | Self::VerticalMomentum
        )
    }
}
