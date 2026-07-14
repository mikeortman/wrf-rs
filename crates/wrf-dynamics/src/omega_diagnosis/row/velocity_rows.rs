#[derive(Clone, Copy)]
pub(crate) struct OmegaDiagnosisVelocityRows<'a> {
    pub(super) west_east: &'a [f32],
    pub(super) east: &'a [f32],
    pub(super) south_north: &'a [f32],
    pub(super) north: &'a [f32],
}

impl<'a> OmegaDiagnosisVelocityRows<'a> {
    pub(crate) fn new(
        west_east: &'a [f32],
        east: &'a [f32],
        south_north: &'a [f32],
        north: &'a [f32],
    ) -> Self {
        let point_count = west_east.len();
        assert_eq!(east.len(), point_count);
        assert_eq!(south_north.len(), point_count);
        assert_eq!(north.len(), point_count);
        Self {
            west_east,
            east,
            south_north,
            north,
        }
    }

    pub(super) const fn point_count(&self) -> usize {
        self.west_east.len()
    }
}
