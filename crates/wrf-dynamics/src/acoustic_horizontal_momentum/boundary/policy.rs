use crate::{
    AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticWestEastBoundary,
    AcousticWestEastPeriodicity,
};

/// Complete lateral boundary policy consumed by WRF `advance_uv`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcousticHorizontalBoundaryPolicy {
    pub(crate) relaxation_zone: AcousticRelaxationZone,
    pub(crate) west_east_periodicity: AcousticWestEastPeriodicity,
    pub(crate) west: AcousticWestEastBoundary,
    pub(crate) east: AcousticWestEastBoundary,
    pub(crate) south: AcousticSouthNorthBoundary,
    pub(crate) north: AcousticSouthNorthBoundary,
}

impl AcousticHorizontalBoundaryPolicy {
    /// Creates a typed replacement for the source configuration booleans.
    pub const fn new(
        relaxation_zone: AcousticRelaxationZone,
        west_east_periodicity: AcousticWestEastPeriodicity,
        west: AcousticWestEastBoundary,
        east: AcousticWestEastBoundary,
        south: AcousticSouthNorthBoundary,
        north: AcousticSouthNorthBoundary,
    ) -> Self {
        Self {
            relaxation_zone,
            west_east_periodicity,
            west,
            east,
            south,
            north,
        }
    }
}
