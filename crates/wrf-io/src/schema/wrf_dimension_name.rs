use crate::WrfIoResult;
use crate::schema::WrfRegistryDimensionName;

/// Names one WRF NetCDF dimension.
///
/// The fixed ARW core dimensions keep dedicated variants; every other
/// Registry-declared dimension is carried as a validated name so schemas are
/// no longer limited to the minimum ARW slice.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    /// Any other Registry-declared dimension.
    Registry(WrfRegistryDimensionName),
}

impl WrfDimensionName {
    /// Returns the exact WRF NetCDF dimension name.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Time => "Time",
            Self::DateStringLength => "DateStrLen",
            Self::WestEast => "west_east",
            Self::SouthNorth => "south_north",
            Self::BottomTop => "bottom_top",
            Self::WestEastStaggered => "west_east_stag",
            Self::SouthNorthStaggered => "south_north_stag",
            Self::BottomTopStaggered => "bottom_top_stag",
            Self::Registry(name) => name.as_str(),
        }
    }

    /// Canonicalizes a file dimension name into its typed form.
    ///
    /// The fixed ARW core names map to their dedicated variants so equality is
    /// representation-independent; any other name is validated as NetCDF
    /// syntax and carried as a Registry dimension.
    pub fn try_from_name(name: &str) -> WrfIoResult<Self> {
        Ok(match name {
            "Time" => Self::Time,
            "DateStrLen" => Self::DateStringLength,
            "west_east" => Self::WestEast,
            "south_north" => Self::SouthNorth,
            "bottom_top" => Self::BottomTop,
            "west_east_stag" => Self::WestEastStaggered,
            "south_north_stag" => Self::SouthNorthStaggered,
            "bottom_top_stag" => Self::BottomTopStaggered,
            other => Self::Registry(WrfRegistryDimensionName::try_new(other)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::WrfIoError;

    use super::*;

    #[test]
    fn try_from_name_canonicalizes_core_names_and_accepts_registry_names() {
        assert_eq!(
            WrfDimensionName::try_from_name("west_east_stag").unwrap(),
            WrfDimensionName::WestEastStaggered
        );
        let registry = WrfDimensionName::try_from_name("soil_layers_stag").unwrap();
        assert!(matches!(registry, WrfDimensionName::Registry(_)));
        assert_eq!(registry.as_str(), "soil_layers_stag");
    }

    #[test]
    fn try_from_name_rejects_invalid_netcdf_syntax() {
        assert!(matches!(
            WrfDimensionName::try_from_name("bad/name"),
            Err(WrfIoError::InvalidDimensionName { .. })
        ));
    }
}
