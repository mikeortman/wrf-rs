/// Borrowed fields participating in one Held-Suarez damping update.
///
/// The bundle makes the two mutable outputs and four immutable inputs explicit
/// without cloning domain-sized storage.
pub struct HeldSuarezDampingFields<'a, Field> {
    pub(crate) west_east_momentum_tendency: &'a mut Field,
    pub(crate) south_north_momentum_tendency: &'a mut Field,
    pub(crate) west_east_momentum: &'a Field,
    pub(crate) south_north_momentum: &'a Field,
    pub(crate) perturbation_pressure: &'a Field,
    pub(crate) base_pressure: &'a Field,
}

impl<'a, Field> HeldSuarezDampingFields<'a, Field> {
    /// Groups all fields for a damping update without allocating or copying.
    pub fn new(
        west_east_momentum_tendency: &'a mut Field,
        south_north_momentum_tendency: &'a mut Field,
        west_east_momentum: &'a Field,
        south_north_momentum: &'a Field,
        perturbation_pressure: &'a Field,
        base_pressure: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum_tendency,
            south_north_momentum_tendency,
            west_east_momentum,
            south_north_momentum,
            perturbation_pressure,
            base_pressure,
        }
    }
}
