/// Names the fixed ARW dimensions supported by the first schema slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WrfDimensionName {
    /// Unlimited output-record dimension.
    Time,
    /// Fixed timestamp character width.
    DateStringLength,
    /// West-east mass points.
    WestEast,
    /// South-north mass points.
    SouthNorth,
    /// Vertical mass levels.
    BottomTop,
    /// West-east velocity points.
    WestEastStaggered,
    /// South-north velocity points.
    SouthNorthStaggered,
    /// Vertical interface levels.
    BottomTopStaggered,
}

impl WrfDimensionName {
    /// Returns the exact WRF NetCDF dimension name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Time => "Time",
            Self::DateStringLength => "DateStrLen",
            Self::WestEast => "west_east",
            Self::SouthNorth => "south_north",
            Self::BottomTop => "bottom_top",
            Self::WestEastStaggered => "west_east_stag",
            Self::SouthNorthStaggered => "south_north_stag",
            Self::BottomTopStaggered => "bottom_top_stag",
        }
    }

    pub(crate) fn try_from_name(name: &str) -> Option<Self> {
        match name {
            "Time" => Some(Self::Time),
            "DateStrLen" => Some(Self::DateStringLength),
            "west_east" => Some(Self::WestEast),
            "south_north" => Some(Self::SouthNorth),
            "bottom_top" => Some(Self::BottomTop),
            "west_east_stag" => Some(Self::WestEastStaggered),
            "south_north_stag" => Some(Self::SouthNorthStaggered),
            "bottom_top_stag" => Some(Self::BottomTopStaggered),
            _ => None,
        }
    }
}
