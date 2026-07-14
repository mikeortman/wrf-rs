use std::ops::Range;

use super::{SpecifiedBoundaryRelaxationCoverage, SpecifiedBoundaryRelaxationRowRanges};
use crate::SpecifiedBoundaryWestEastPeriodicity;
use crate::specified_boundary_update::region::SpecifiedBoundaryActiveRanges;

/// Exact half-open translation of WRF's four relaxation-zone loop nests.
pub(in crate::specified_boundary_update::relaxation) struct SpecifiedBoundaryRelaxationRanges {
    ranges: SpecifiedBoundaryActiveRanges,
    specified_zone_width: usize,
    relaxation_zone_width: usize,
    periodic_west_east: bool,
}

impl SpecifiedBoundaryRelaxationRanges {
    pub(in crate::specified_boundary_update::relaxation) fn new(
        ranges: SpecifiedBoundaryActiveRanges,
        specified_zone_width: usize,
        relaxation_zone_width: usize,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    ) -> Self {
        Self {
            ranges,
            specified_zone_width,
            relaxation_zone_width,
            periodic_west_east: west_east_periodicity.is_periodic(),
        }
    }

    pub(in crate::specified_boundary_update::relaxation) fn bottom_top_range(
        &self,
    ) -> Range<usize> {
        self.ranges.bottom_top.clone()
    }

    pub(in crate::specified_boundary_update::relaxation) fn west_east_domain(
        &self,
    ) -> Range<usize> {
        self.ranges.effective_west_east_domain.clone()
    }

    pub(in crate::specified_boundary_update::relaxation) fn south_north_domain(
        &self,
    ) -> Range<usize> {
        self.ranges.effective_south_north_domain.clone()
    }

    pub(in crate::specified_boundary_update::relaxation) fn ranges_for_row(
        &self,
        south_north: usize,
    ) -> SpecifiedBoundaryRelaxationRowRanges {
        if !self.ranges.south_north.contains(&south_north) {
            return SpecifiedBoundaryRelaxationRowRanges::empty();
        }
        SpecifiedBoundaryRelaxationRowRanges {
            south: self.south_range(south_north),
            north: self.north_range(south_north),
            west: self.west_range(south_north),
            east: self.east_range(south_north),
        }
    }

    pub(in crate::specified_boundary_update::relaxation) fn required_field_coverage(
        &self,
    ) -> Option<SpecifiedBoundaryRelaxationCoverage> {
        let mut minimum_west_east = usize::MAX;
        let mut maximum_west_east = 0;
        let mut minimum_south_north = usize::MAX;
        let mut maximum_south_north = 0;

        for south_north in self.ranges.south_north.clone() {
            for range in self.ranges_for_row(south_north).iter() {
                minimum_west_east = minimum_west_east.min(range.start);
                maximum_west_east = maximum_west_east.max(range.end);
                minimum_south_north = minimum_south_north.min(south_north);
                maximum_south_north = maximum_south_north.max(south_north + 1);
            }
        }
        if minimum_west_east == usize::MAX {
            return None;
        }

        Some(SpecifiedBoundaryRelaxationCoverage {
            west_east: minimum_west_east
                .saturating_sub(1)
                .max(self.ranges.effective_west_east_domain.start)
                ..maximum_west_east
                    .saturating_add(1)
                    .min(self.ranges.effective_west_east_domain.end),
            south_north: minimum_south_north
                .saturating_sub(1)
                .max(self.ranges.effective_south_north_domain.start)
                ..maximum_south_north
                    .saturating_add(1)
                    .min(self.ranges.effective_south_north_domain.end),
            bottom_top: self.ranges.bottom_top.clone(),
        })
    }

    fn south_range(&self, south_north: usize) -> Option<Range<usize>> {
        if !self.touches_south_boundary() {
            return None;
        }
        let start = self
            .ranges
            .effective_south_north_domain
            .start
            .saturating_add(self.specified_zone_width);
        let end = self
            .ranges
            .effective_south_north_domain
            .start
            .saturating_add(self.relaxation_zone_width)
            .min(self.ranges.effective_south_north_domain.end);
        if !(start..end).contains(&south_north) {
            return None;
        }
        let distance = south_north - self.ranges.effective_south_north_domain.start;
        self.horizontal_trapezoid_range(distance)
    }

    fn north_range(&self, south_north: usize) -> Option<Range<usize>> {
        if !self.touches_north_boundary() {
            return None;
        }
        let start = self
            .ranges
            .effective_south_north_domain
            .end
            .saturating_sub(self.relaxation_zone_width)
            .max(self.ranges.effective_south_north_domain.start);
        let end = self
            .ranges
            .effective_south_north_domain
            .end
            .saturating_sub(self.specified_zone_width);
        if !(start..end).contains(&south_north) {
            return None;
        }
        let distance = self.ranges.effective_south_north_domain.end - 1 - south_north;
        self.horizontal_trapezoid_range(distance)
    }

    fn horizontal_trapezoid_range(&self, distance: usize) -> Option<Range<usize>> {
        let corner_limit = if self.periodic_west_east { 0 } else { distance };
        nonempty(
            self.ranges.west_east.start.max(
                self.ranges
                    .effective_west_east_domain
                    .start
                    .saturating_add(corner_limit),
            )
                ..self.ranges.west_east.end.min(
                    self.ranges
                        .effective_west_east_domain
                        .end
                        .saturating_sub(corner_limit),
                ),
        )
    }

    fn west_range(&self, south_north: usize) -> Option<Range<usize>> {
        if self.periodic_west_east || !self.touches_west_boundary() {
            return None;
        }
        let maximum_distance = self.maximum_side_distance(south_north)?;
        nonempty(
            self.ranges.west_east.start.max(
                self.ranges
                    .effective_west_east_domain
                    .start
                    .saturating_add(self.specified_zone_width),
            )
                ..self.ranges.west_east.end.min(
                    self.ranges
                        .effective_west_east_domain
                        .start
                        .saturating_add(self.relaxation_zone_width)
                        .min(
                            self.ranges
                                .effective_west_east_domain
                                .start
                                .saturating_add(maximum_distance)
                                .saturating_add(1),
                        ),
                ),
        )
    }

    fn east_range(&self, south_north: usize) -> Option<Range<usize>> {
        if self.periodic_west_east || !self.touches_east_boundary() {
            return None;
        }
        let maximum_distance = self.maximum_side_distance(south_north)?;
        nonempty(
            self.ranges.west_east.start.max(
                self.ranges
                    .effective_west_east_domain
                    .end
                    .saturating_sub(self.relaxation_zone_width)
                    .max(
                        self.ranges
                            .effective_west_east_domain
                            .end
                            .saturating_sub(maximum_distance.saturating_add(1)),
                    ),
            )
                ..self.ranges.west_east.end.min(
                    self.ranges
                        .effective_west_east_domain
                        .end
                        .saturating_sub(self.specified_zone_width),
                ),
        )
    }

    fn maximum_side_distance(&self, south_north: usize) -> Option<usize> {
        let south_distance = south_north
            .checked_sub(self.ranges.effective_south_north_domain.start)?
            .checked_sub(1)?;
        let north_distance = self
            .ranges
            .effective_south_north_domain
            .end
            .checked_sub(south_north)?
            .checked_sub(2)?;
        Some(south_distance.min(north_distance))
    }

    fn touches_south_boundary(&self) -> bool {
        self.ranges.south_north.start - self.ranges.effective_south_north_domain.start
            < self.relaxation_zone_width
    }

    fn touches_north_boundary(&self) -> bool {
        self.ranges.effective_south_north_domain.end - self.ranges.south_north.end
            < self.relaxation_zone_width
    }

    fn touches_west_boundary(&self) -> bool {
        self.ranges.west_east.start - self.ranges.effective_west_east_domain.start
            < self.relaxation_zone_width
    }

    fn touches_east_boundary(&self) -> bool {
        self.ranges.effective_west_east_domain.end - self.ranges.west_east.end
            < self.relaxation_zone_width
    }
}

fn nonempty(range: Range<usize>) -> Option<Range<usize>> {
    (!range.is_empty()).then_some(range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_open_ranges_match_source_loop_membership() {
        let domain = 1..9;
        let tiles = [1..9, 1..6, 4..9, 3..7];

        for west_east in &tiles {
            for south_north in &tiles {
                for periodicity in [
                    SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                    SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ] {
                    let ranges = SpecifiedBoundaryRelaxationRanges::new(
                        SpecifiedBoundaryActiveRanges {
                            west_east: west_east.clone(),
                            south_north: south_north.clone(),
                            bottom_top: 1..3,
                            effective_west_east_domain: domain.clone(),
                            effective_south_north_domain: domain.clone(),
                        },
                        1,
                        4,
                        periodicity,
                    );
                    for row in 0..10 {
                        let actual = ranges.ranges_for_row(row);
                        for column in 0..10 {
                            let actual_count = actual
                                .iter()
                                .filter(|range| range.contains(&column))
                                .count();
                            assert_eq!(
                                actual_count,
                                source_update_count(
                                    column,
                                    row,
                                    west_east,
                                    south_north,
                                    &domain,
                                    periodicity.is_periodic(),
                                ),
                                "column={column}, row={row}, x_tile={west_east:?}, y_tile={south_north:?}, periodicity={periodicity:?}",
                            );
                        }
                    }
                }
            }
        }
    }

    fn source_update_count(
        column: usize,
        row: usize,
        west_east: &Range<usize>,
        south_north: &Range<usize>,
        domain: &Range<usize>,
        periodic_west_east: bool,
    ) -> usize {
        if !west_east.contains(&column) || !south_north.contains(&row) {
            return 0;
        }
        let south_distance = row.saturating_sub(domain.start);
        let north_distance = domain.end.saturating_sub(1).saturating_sub(row);
        let south_limit = if periodic_west_east {
            0
        } else {
            south_distance
        };
        let north_limit = if periodic_west_east {
            0
        } else {
            north_distance
        };
        let south = south_north.start - domain.start < 4
            && (domain.start + 1..domain.start + 4).contains(&row)
            && (domain.start + south_limit..domain.end.saturating_sub(south_limit))
                .contains(&column);
        let north = domain.end - south_north.end < 4
            && (domain.end - 4..domain.end - 1).contains(&row)
            && (domain.start + north_limit..domain.end.saturating_sub(north_limit))
                .contains(&column);
        let west_distance = column.saturating_sub(domain.start);
        let east_distance = domain.end.saturating_sub(1).saturating_sub(column);
        let west = !periodic_west_east
            && west_east.start - domain.start < 4
            && (domain.start + 1..domain.start + 4).contains(&column)
            && (domain.start + west_distance + 1..domain.end.saturating_sub(west_distance + 1))
                .contains(&row);
        let east = !periodic_west_east
            && domain.end - west_east.end < 4
            && (domain.end - 4..domain.end - 1).contains(&column)
            && (domain.start + east_distance + 1..domain.end.saturating_sub(east_distance + 1))
                .contains(&row);
        usize::from(south) + usize::from(north) + usize::from(west) + usize::from(east)
    }
}
