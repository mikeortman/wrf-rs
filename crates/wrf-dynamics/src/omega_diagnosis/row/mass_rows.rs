#[derive(Clone, Copy)]
pub(crate) struct OmegaDiagnosisMassRows<'a> {
    pub(super) perturbation_current: &'a [f32],
    pub(super) perturbation_west: &'a [f32],
    pub(super) perturbation_east: &'a [f32],
    pub(super) perturbation_south: &'a [f32],
    pub(super) perturbation_north: &'a [f32],
    pub(super) base_current: &'a [f32],
    pub(super) base_west: &'a [f32],
    pub(super) base_east: &'a [f32],
    pub(super) base_south: &'a [f32],
    pub(super) base_north: &'a [f32],
}

impl<'a> OmegaDiagnosisMassRows<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        perturbation_current: &'a [f32],
        perturbation_west: &'a [f32],
        perturbation_east: &'a [f32],
        perturbation_south: &'a [f32],
        perturbation_north: &'a [f32],
        base_current: &'a [f32],
        base_west: &'a [f32],
        base_east: &'a [f32],
        base_south: &'a [f32],
        base_north: &'a [f32],
    ) -> Self {
        let point_count = perturbation_current.len();
        for row in [
            perturbation_west,
            perturbation_east,
            perturbation_south,
            perturbation_north,
            base_current,
            base_west,
            base_east,
            base_south,
            base_north,
        ] {
            assert_eq!(row.len(), point_count);
        }
        Self {
            perturbation_current,
            perturbation_west,
            perturbation_east,
            perturbation_south,
            perturbation_north,
            base_current,
            base_west,
            base_east,
            base_south,
            base_north,
        }
    }

    pub(super) const fn point_count(&self) -> usize {
        self.perturbation_current.len()
    }
}
