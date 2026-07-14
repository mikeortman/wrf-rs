use std::ops::Range;

use super::SpecifiedBoundaryRowRanges;
use crate::SpecifiedBoundaryWestEastPeriodicity;
use crate::specified_boundary_update::region::SpecifiedBoundaryActiveRanges;

/// Direct WRF trapezoid and side ranges shared by specified-boundary kernels.
pub(crate) struct SpecifiedBoundaryRanges {
    ranges: SpecifiedBoundaryActiveRanges,
    specified_zone_width: usize,
    periodic_west_east: bool,
}

impl SpecifiedBoundaryRanges {
    pub(crate) fn new(
        ranges: SpecifiedBoundaryActiveRanges,
        specified_zone_width: usize,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    ) -> Self {
        Self {
            ranges,
            specified_zone_width,
            periodic_west_east: west_east_periodicity.is_periodic(),
        }
    }

    pub(crate) fn bottom_top_range(&self) -> Range<usize> {
        self.ranges.bottom_top.clone()
    }

    pub(crate) fn west_east_domain(&self) -> Range<usize> {
        self.ranges.effective_west_east_domain.clone()
    }

    pub(crate) fn south_north_domain(&self) -> Range<usize> {
        self.ranges.effective_south_north_domain.clone()
    }

    pub(crate) fn ranges_for_row(&self, south_north: usize) -> SpecifiedBoundaryRowRanges {
        if !self.ranges.south_north.contains(&south_north) {
            return SpecifiedBoundaryRowRanges::empty();
        }
        SpecifiedBoundaryRowRanges {
            south: self.south_range(south_north),
            north: self.north_range(south_north),
            west: self.west_range(south_north),
            east: self.east_range(south_north),
        }
    }

    fn south_range(&self, south_north: usize) -> Option<Range<usize>> {
        if !self.touches_south_boundary()
            || south_north
                >= self
                    .ranges
                    .effective_south_north_domain
                    .start
                    .saturating_add(self.specified_zone_width)
        {
            return None;
        }
        let distance = south_north - self.ranges.effective_south_north_domain.start;
        self.horizontal_trapezoid_range(distance)
    }

    fn north_range(&self, south_north: usize) -> Option<Range<usize>> {
        if !self.touches_north_boundary()
            || south_north
                < self
                    .ranges
                    .effective_south_north_domain
                    .end
                    .saturating_sub(self.specified_zone_width)
        {
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
            self.ranges.west_east.start
                ..self.ranges.west_east.end.min(
                    self.ranges
                        .effective_west_east_domain
                        .start
                        .saturating_add(self.specified_zone_width)
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
                    .saturating_sub(self.specified_zone_width)
                    .max(
                        self.ranges
                            .effective_west_east_domain
                            .end
                            .saturating_sub(maximum_distance.saturating_add(1)),
                    ),
            )..self.ranges.west_east.end,
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
            < self.specified_zone_width
    }

    fn touches_north_boundary(&self) -> bool {
        self.ranges.effective_south_north_domain.end - self.ranges.south_north.end
            < self.specified_zone_width
    }

    fn touches_west_boundary(&self) -> bool {
        self.ranges.west_east.start - self.ranges.effective_west_east_domain.start
            < self.specified_zone_width
    }

    fn touches_east_boundary(&self) -> bool {
        self.ranges.effective_west_east_domain.end - self.ranges.west_east.end
            < self.specified_zone_width
    }
}

fn nonempty(range: Range<usize>) -> Option<Range<usize>> {
    (!range.is_empty()).then_some(range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_ranges_match_source_loop_membership_for_tiles_zones_and_periodicity() {
        let domain = 1..6;
        let tiles = [1..6, 1..4, 3..6, 2..5];
        for west_east in &tiles {
            for south_north in &tiles {
                for specified_zone_width in 0..=8 {
                    for periodicity in [
                        SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
                        SpecifiedBoundaryWestEastPeriodicity::Periodic,
                    ] {
                        let boundary_ranges = SpecifiedBoundaryRanges::new(
                            SpecifiedBoundaryActiveRanges {
                                west_east: west_east.clone(),
                                south_north: south_north.clone(),
                                bottom_top: 1..2,
                                effective_west_east_domain: domain.clone(),
                                effective_south_north_domain: domain.clone(),
                            },
                            specified_zone_width,
                            periodicity,
                        );
                        for row in 0..7 {
                            let ranges = boundary_ranges.ranges_for_row(row);
                            for column in 0..7 {
                                let actual = [
                                    ranges.south.as_ref(),
                                    ranges.north.as_ref(),
                                    ranges.west.as_ref(),
                                    ranges.east.as_ref(),
                                ]
                                .into_iter()
                                .flatten()
                                .filter(|range| range.contains(&column))
                                .count();
                                assert_eq!(
                                    actual,
                                    source_update_count(
                                        column,
                                        row,
                                        west_east,
                                        south_north,
                                        &domain,
                                        specified_zone_width,
                                        periodicity.is_periodic(),
                                    ),
                                    "column={column}, row={row}, x_tile={west_east:?}, y_tile={south_north:?}, zone={specified_zone_width}, periodicity={periodicity:?}",
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn source_update_count(
        column: usize,
        row: usize,
        west_east: &Range<usize>,
        south_north: &Range<usize>,
        domain: &Range<usize>,
        specified_zone_width: usize,
        periodic_west_east: bool,
    ) -> usize {
        if !west_east.contains(&column) || !south_north.contains(&row) {
            return 0;
        }
        let touches_south = south_north.start - domain.start < specified_zone_width;
        let touches_north = domain.end - south_north.end < specified_zone_width;
        let touches_west = west_east.start - domain.start < specified_zone_width;
        let touches_east = domain.end - west_east.end < specified_zone_width;
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
        let south = touches_south
            && row < domain.start.saturating_add(specified_zone_width)
            && (domain.start.saturating_add(south_limit)..domain.end.saturating_sub(south_limit))
                .contains(&column);
        let north = touches_north
            && row >= domain.end.saturating_sub(specified_zone_width)
            && (domain.start.saturating_add(north_limit)..domain.end.saturating_sub(north_limit))
                .contains(&column);
        let west_distance = column.saturating_sub(domain.start);
        let east_distance = domain.end.saturating_sub(1).saturating_sub(column);
        let west = !periodic_west_east
            && touches_west
            && column < domain.start.saturating_add(specified_zone_width)
            && (domain.start.saturating_add(west_distance).saturating_add(1)
                ..domain.end.saturating_sub(west_distance.saturating_add(1)))
                .contains(&row);
        let east = !periodic_west_east
            && touches_east
            && column >= domain.end.saturating_sub(specified_zone_width)
            && (domain.start.saturating_add(east_distance).saturating_add(1)
                ..domain.end.saturating_sub(east_distance.saturating_add(1)))
                .contains(&row);
        usize::from(south) + usize::from(north) + usize::from(west) + usize::from(east)
    }
}
