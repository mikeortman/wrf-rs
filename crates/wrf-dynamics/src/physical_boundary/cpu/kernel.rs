use super::geometry::PhysicalBoundaryGeometry;

/// Applies every west-east branch of `set_physical_bc3d` to one `j` plane.
///
/// The plane is the contiguous `(i, k)` slab for row `j`; reads and writes
/// stay inside the plane, which keeps row-parallel execution bitwise equal to
/// WRF's serial loops.
pub(super) fn apply_volume_west_east(
    plane: &mut [f32],
    row: isize,
    geometry: &PhysicalBoundaryGeometry,
) {
    let stride = geometry.west_east_points;
    let index = |i: isize, k: isize| (k * stride + i) as usize;
    let conditions = geometry.conditions;
    let zone = geometry.zone;
    let (window_start, window_end) = geometry.lateral_row_window();
    let in_window = row >= window_start && row <= window_end;

    if conditions.periodic_x {
        // Single-rank patches always satisfy WRF's on-processor test
        // (ids == ips .and. ide == ipe).
        if !in_window {
            return;
        }
        if geometry.touches_west() {
            for level in geometry.kts..=geometry.k_end {
                for offset in 0..zone {
                    plane[index(geometry.ids - 1 - offset, level)] =
                        plane[index(geometry.ide - 1 - offset, level)];
                }
            }
        }
        if geometry.touches_east() {
            let stagger = geometry.west_east_stagger;
            for level in geometry.kts..=geometry.k_end {
                for offset in -stagger..=zone {
                    plane[index(geometry.ide + offset + stagger, level)] =
                        plane[index(geometry.ids + offset + stagger, level)];
                }
            }
        }
        return;
    }

    if conditions.symmetric_xs && geometry.touches_west() && in_window {
        for level in geometry.kts..=geometry.k_end {
            if geometry.west_east_stagger == -1 {
                for offset in 1..=zone {
                    plane[index(geometry.ids - offset, level)] =
                        plane[index(geometry.ids + offset - 1, level)];
                }
            } else if geometry.variable.is_west_east_face() {
                for offset in 1..=zone {
                    plane[index(geometry.ids - offset, level)] =
                        -plane[index(geometry.ids + offset, level)];
                }
            } else {
                for offset in 1..=zone {
                    plane[index(geometry.ids - offset, level)] =
                        plane[index(geometry.ids + offset, level)];
                }
            }
        }
    }

    if conditions.symmetric_xe && geometry.touches_east() && in_window {
        for level in geometry.kts..=geometry.k_end {
            if geometry.west_east_stagger == -1 {
                for offset in 1..=zone {
                    plane[index(geometry.ide + offset - 1, level)] =
                        plane[index(geometry.ide - offset, level)];
                }
            } else if geometry.variable.is_west_east_face() {
                for offset in 1..=zone {
                    plane[index(geometry.ide + offset, level)] =
                        -plane[index(geometry.ide - offset, level)];
                }
            } else {
                for offset in 1..=zone {
                    plane[index(geometry.ide + offset, level)] =
                        plane[index(geometry.ide - offset, level)];
                }
            }
        }
    }

    let open_row_end = geometry
        .jte
        .min(geometry.jde + geometry.south_north_stagger)
        + zone;
    if conditions.copies_open_west()
        && geometry.touches_west()
        && row >= geometry.jts - zone
        && row <= open_row_end
    {
        for level in geometry.kts..=geometry.k_end {
            let edge = plane[index(geometry.ids, level)];
            plane[index(geometry.ids - 1, level)] = edge;
            plane[index(geometry.ids - 2, level)] = edge;
            plane[index(geometry.ids - 3, level)] = edge;
        }
    }

    if conditions.copies_open_east() && geometry.touches_east() {
        if geometry.variable.is_west_east_face() {
            let staggered_row_start = geometry.jds.max(geometry.jts - 1) - zone;
            let staggered_row_end =
                (geometry.jte + 1).min(geometry.jde + geometry.south_north_stagger) + zone;
            if row >= staggered_row_start && row <= staggered_row_end {
                for level in geometry.kts..=geometry.k_end {
                    let edge = plane[index(geometry.ide, level)];
                    plane[index(geometry.ide + 1, level)] = edge;
                    plane[index(geometry.ide + 2, level)] = edge;
                    plane[index(geometry.ide + 3, level)] = edge;
                }
            }
        } else if row >= geometry.jts - zone && row <= open_row_end {
            for level in geometry.kts..=geometry.k_end {
                let interior = plane[index(geometry.ide - 1, level)];
                plane[index(geometry.ide, level)] = interior;
                plane[index(geometry.ide + 1, level)] = interior;
                plane[index(geometry.ide + 2, level)] = interior;
            }
        }
    }
}

/// Applies every south-north branch of `set_physical_bc3d` sequentially.
///
/// The copies read rows that earlier iterations may have written, so the
/// Fortran iteration order is preserved exactly instead of parallelizing.
pub(super) fn apply_volume_south_north(values: &mut [f32], geometry: &PhysicalBoundaryGeometry) {
    let row_stride = geometry.west_east_points;
    let plane_stride = row_stride * geometry.bottom_top_points;
    let index = |i: isize, k: isize, j: isize| (j * plane_stride + k * row_stride + i) as usize;
    copy_south_north(values, geometry, index);
}

/// Applies every west-east branch of `set_physical_bc2d` to one `j` row.
pub(super) fn apply_horizontal_west_east(
    row_values: &mut [f32],
    row: isize,
    geometry: &PhysicalBoundaryGeometry,
) {
    let conditions = geometry.conditions;
    let zone = geometry.zone;
    let (window_start, window_end) = geometry.lateral_row_window();
    if row < window_start || row > window_end {
        return;
    }
    let index = |i: isize| i as usize;

    if conditions.periodic_x {
        if geometry.touches_west() {
            for offset in 0..zone {
                row_values[index(geometry.ids - 1 - offset)] =
                    row_values[index(geometry.ide - 1 - offset)];
            }
        }
        if geometry.touches_east() {
            let stagger = geometry.west_east_stagger;
            for offset in -stagger..=zone {
                row_values[index(geometry.ide + offset + stagger)] =
                    row_values[index(geometry.ids + offset + stagger)];
            }
        }
        return;
    }

    if conditions.symmetric_xs && geometry.touches_west() {
        if geometry.west_east_stagger == -1 {
            for offset in 1..=zone {
                row_values[index(geometry.ids - offset)] =
                    row_values[index(geometry.ids + offset - 1)];
            }
        } else if geometry.variable.is_west_east_face() {
            // WRF's two-dimensional staggered reflection starts at the edge
            // point itself (DO i = 0, bdyzone-1), unlike the volume routine.
            for offset in 0..zone {
                row_values[index(geometry.ids - offset)] =
                    -row_values[index(geometry.ids + offset)];
            }
        } else {
            for offset in 0..zone {
                row_values[index(geometry.ids - offset)] = row_values[index(geometry.ids + offset)];
            }
        }
    }

    if conditions.symmetric_xe && geometry.touches_east() {
        if geometry.west_east_stagger == -1 {
            for offset in 1..=zone {
                row_values[index(geometry.ide + offset - 1)] =
                    row_values[index(geometry.ide - offset)];
            }
        } else if geometry.variable.is_west_east_face() {
            for offset in 0..zone {
                row_values[index(geometry.ide + offset)] =
                    -row_values[index(geometry.ide - offset)];
            }
        } else {
            for offset in 0..zone {
                row_values[index(geometry.ide + offset)] = row_values[index(geometry.ide - offset)];
            }
        }
    }

    if conditions.copies_open_west() && geometry.touches_west() {
        let edge = row_values[index(geometry.ids)];
        row_values[index(geometry.ids - 1)] = edge;
        row_values[index(geometry.ids - 2)] = edge;
        row_values[index(geometry.ids - 3)] = edge;
    }

    if conditions.copies_open_east() && geometry.touches_east() {
        if geometry.variable.is_west_east_face() {
            let edge = row_values[index(geometry.ide)];
            row_values[index(geometry.ide + 1)] = edge;
            row_values[index(geometry.ide + 2)] = edge;
            row_values[index(geometry.ide + 3)] = edge;
        } else {
            let interior = row_values[index(geometry.ide - 1)];
            row_values[index(geometry.ide)] = interior;
            row_values[index(geometry.ide + 1)] = interior;
            row_values[index(geometry.ide + 2)] = interior;
        }
    }
}

/// Applies the south-north branches and doubly periodic corner fills of
/// `set_physical_bc2d` sequentially in Fortran order.
pub(super) fn apply_horizontal_south_north(
    values: &mut [f32],
    geometry: &PhysicalBoundaryGeometry,
) {
    let row_stride = geometry.west_east_points;
    let index = |i: isize, _k: isize, j: isize| (j * row_stride + i) as usize;
    copy_south_north(values, geometry, index);

    let conditions = geometry.conditions;
    if !(conditions.periodic_x && conditions.periodic_y) {
        return;
    }
    let corner = |i: isize, j: isize| (j * row_stride + i) as usize;
    let zone = geometry.zone;
    let west_east_stagger = geometry.west_east_stagger;
    let south_north_stagger = geometry.south_north_stagger;
    if geometry.touches_west() && geometry.touches_south() {
        for row_offset in 0..zone {
            for column_offset in 0..zone {
                values[corner(
                    geometry.ids - 1 - column_offset,
                    geometry.jds - 1 - row_offset,
                )] = values[corner(
                    geometry.ide - 1 - column_offset,
                    geometry.jde - 1 - row_offset,
                )];
            }
        }
    }
    if geometry.touches_east() && geometry.touches_south() {
        for row_offset in 0..zone {
            for column_offset in 1..=zone {
                values[corner(
                    geometry.ide + column_offset + west_east_stagger,
                    geometry.jds - 1 - row_offset,
                )] = values[corner(
                    geometry.ids + column_offset + west_east_stagger,
                    geometry.jde - 1 - row_offset,
                )];
            }
        }
    }
    if geometry.touches_east() && geometry.touches_north() {
        for row_offset in 1..=zone {
            for column_offset in 1..=zone {
                values[corner(
                    geometry.ide + column_offset + west_east_stagger,
                    geometry.jde + row_offset + south_north_stagger,
                )] = values[corner(
                    geometry.ids + column_offset + west_east_stagger,
                    geometry.jds + row_offset + south_north_stagger,
                )];
            }
        }
    }
    if geometry.touches_west() && geometry.touches_north() {
        for row_offset in 1..=zone {
            for column_offset in 0..zone {
                values[corner(
                    geometry.ids - 1 - column_offset,
                    geometry.jde + row_offset + south_north_stagger,
                )] = values[corner(
                    geometry.ide - 1 - column_offset,
                    geometry.jds + row_offset + south_north_stagger,
                )];
            }
        }
    }
}

/// Shared south-north copy loops used by the volume and horizontal kernels.
///
/// `index` resolves `(i, k, j)` in the caller's storage; the horizontal
/// caller pins `k` to zero via `kts ..= k_end` collapsing to one level.
fn copy_south_north<IndexFn>(
    values: &mut [f32],
    geometry: &PhysicalBoundaryGeometry,
    index: IndexFn,
) where
    IndexFn: Fn(isize, isize, isize) -> usize,
{
    let conditions = geometry.conditions;
    let zone = geometry.zone;
    let (column_start, column_end) = geometry.south_north_column_window();
    let levels = geometry.kts..=geometry.k_end;

    if conditions.periodic_y {
        // Single-rank patches always satisfy jds == jps .and. jde == jpe.
        if geometry.touches_south() {
            for row_offset in 0..zone {
                for level in levels.clone() {
                    for column in column_start..=column_end {
                        values[index(column, level, geometry.jds - 1 - row_offset)] =
                            values[index(column, level, geometry.jde - 1 - row_offset)];
                    }
                }
            }
        }
        if geometry.touches_north() {
            let stagger = geometry.south_north_stagger;
            for row_offset in -stagger..=zone {
                for level in levels.clone() {
                    for column in column_start..=column_end {
                        values[index(column, level, geometry.jde + row_offset + stagger)] =
                            values[index(column, level, geometry.jds + row_offset + stagger)];
                    }
                }
            }
        }
        return;
    }

    if conditions.symmetric_ys && geometry.touches_south() {
        for row_offset in 1..=zone {
            for level in levels.clone() {
                for column in column_start..=column_end {
                    let source = if geometry.south_north_stagger == -1 {
                        values[index(column, level, geometry.jds + row_offset - 1)]
                    } else if geometry.variable.is_south_north_face() {
                        -values[index(column, level, geometry.jds + row_offset)]
                    } else {
                        values[index(column, level, geometry.jds + row_offset)]
                    };
                    values[index(column, level, geometry.jds - row_offset)] = source;
                }
            }
        }
    }

    if conditions.symmetric_ye && geometry.touches_north() {
        for row_offset in 1..=zone {
            for level in levels.clone() {
                for column in column_start..=column_end {
                    if geometry.south_north_stagger == -1 {
                        values[index(column, level, geometry.jde + row_offset - 1)] =
                            values[index(column, level, geometry.jde - row_offset)];
                    } else if geometry.variable.is_south_north_face() {
                        values[index(column, level, geometry.jde + row_offset)] =
                            -values[index(column, level, geometry.jde - row_offset)];
                    } else {
                        values[index(column, level, geometry.jde + row_offset)] =
                            values[index(column, level, geometry.jde - row_offset)];
                    }
                }
            }
        }
    }

    if conditions.copies_open_south() && geometry.touches_south() {
        for level in levels.clone() {
            for column in column_start..=column_end {
                let edge = values[index(column, level, geometry.jds)];
                values[index(column, level, geometry.jds - 1)] = edge;
                values[index(column, level, geometry.jds - 2)] = edge;
                values[index(column, level, geometry.jds - 3)] = edge;
            }
        }
    }

    if conditions.copies_open_north() && geometry.touches_north() {
        if geometry.variable.is_south_north_face() {
            for level in levels.clone() {
                for column in column_start..=column_end {
                    let edge = values[index(column, level, geometry.jde)];
                    values[index(column, level, geometry.jde + 1)] = edge;
                    values[index(column, level, geometry.jde + 2)] = edge;
                    values[index(column, level, geometry.jde + 3)] = edge;
                }
            }
        } else {
            for level in levels.clone() {
                for column in column_start..=column_end {
                    let interior = values[index(column, level, geometry.jde - 1)];
                    values[index(column, level, geometry.jde)] = interior;
                    values[index(column, level, geometry.jde + 1)] = interior;
                    values[index(column, level, geometry.jde + 2)] = interior;
                }
            }
        }
    }
}
