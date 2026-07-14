/// Periodic horizontal axes used by WRF's big-step mass staggering.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColumnMassStaggeringPeriodicity {
    /// Neither horizontal axis is periodic.
    #[default]
    None,
    /// Only the west-east axis is periodic.
    WestEast,
    /// Only the south-north axis is periodic.
    SouthNorth,
    /// Both horizontal axes are periodic.
    Both,
}

impl ColumnMassStaggeringPeriodicity {
    /// Returns whether west-east boundary points use periodic halo mass.
    pub const fn is_west_east_periodic(self) -> bool {
        matches!(self, Self::WestEast | Self::Both)
    }

    /// Returns whether south-north boundary points use periodic halo mass.
    pub const fn is_south_north_periodic(self) -> bool {
        matches!(self, Self::SouthNorth | Self::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_select_only_their_named_periodic_axes() {
        assert!(!ColumnMassStaggeringPeriodicity::None.is_west_east_periodic());
        assert!(!ColumnMassStaggeringPeriodicity::None.is_south_north_periodic());
        assert!(ColumnMassStaggeringPeriodicity::WestEast.is_west_east_periodic());
        assert!(!ColumnMassStaggeringPeriodicity::WestEast.is_south_north_periodic());
        assert!(!ColumnMassStaggeringPeriodicity::SouthNorth.is_west_east_periodic());
        assert!(ColumnMassStaggeringPeriodicity::SouthNorth.is_south_north_periodic());
        assert!(ColumnMassStaggeringPeriodicity::Both.is_west_east_periodic());
        assert!(ColumnMassStaggeringPeriodicity::Both.is_south_north_periodic());
    }
}
