pub(crate) fn clamp_to_interior(value: usize, lower: usize, upper: usize) -> usize {
    value.max(lower).min(upper)
}

pub(crate) fn volume_index(
    west_east: usize,
    bottom_top: usize,
    south_north: usize,
    west_east_points: usize,
    bottom_top_points: usize,
) -> usize {
    west_east + west_east_points * (bottom_top + bottom_top_points * south_north)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interior_clamping_preserves_core_points_and_clamps_both_sides() {
        assert_eq!(clamp_to_interior(1, 3, 6), 3);
        assert_eq!(clamp_to_interior(4, 3, 6), 4);
        assert_eq!(clamp_to_interior(8, 3, 6), 6);
    }

    #[test]
    fn volume_index_matches_wrf_xzy_storage_order() {
        assert_eq!(volume_index(2, 3, 4, 8, 9), 314);
    }
}
