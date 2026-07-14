/// Mutable moisture coefficients on the three ARW momentum staggers.
pub struct MoistureCoefficientOutputs<'a, Field> {
    pub(crate) west_east: &'a mut Field,
    pub(crate) south_north: &'a mut Field,
    pub(crate) vertical: &'a mut Field,
}

impl<'a, Field> MoistureCoefficientOutputs<'a, Field> {
    /// Groups `cqu`, `cqv`, and `cqw` without allocating or permitting aliasing.
    pub fn new(
        west_east: &'a mut Field,
        south_north: &'a mut Field,
        vertical: &'a mut Field,
    ) -> Self {
        Self {
            west_east,
            south_north,
            vertical,
        }
    }
}
