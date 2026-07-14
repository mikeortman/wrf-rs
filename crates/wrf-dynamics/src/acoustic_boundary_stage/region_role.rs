use std::fmt;

/// Scientific role of a location-specific specified-boundary region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticBoundaryRegionRole {
    /// West-east staggered momentum.
    WestEastMomentum,
    /// South-north staggered momentum.
    SouthNorthMomentum,
    /// Half-level potential temperature.
    PotentialTemperature,
    /// Single-level perturbation and coupled column mass.
    ColumnMass,
    /// Full-level vertical momentum and geopotential.
    FullLevel,
}

impl fmt::Display for AcousticBoundaryRegionRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEastMomentum => formatter.write_str("west-east momentum"),
            Self::SouthNorthMomentum => formatter.write_str("south-north momentum"),
            Self::PotentialTemperature => formatter.write_str("potential temperature"),
            Self::ColumnMass => formatter.write_str("column mass"),
            Self::FullLevel => formatter.write_str("full-level state"),
        }
    }
}
