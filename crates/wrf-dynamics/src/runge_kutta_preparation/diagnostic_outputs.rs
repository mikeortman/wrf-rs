/// Mutable scalar diagnostic outputs produced after momentum coupling.
pub struct RungeKuttaPreparationDiagnosticOutputs<'a, Field> {
    pub(crate) omega: &'a mut Field,
    pub(crate) west_east_moisture: &'a mut Field,
    pub(crate) south_north_moisture: &'a mut Field,
    pub(crate) vertical_moisture: &'a mut Field,
    pub(crate) full_inverse_density: &'a mut Field,
    pub(crate) pressure_point_geopotential: &'a mut Field,
}

impl<'a, Field> RungeKuttaPreparationDiagnosticOutputs<'a, Field> {
    /// Groups WRF `ww`, `cqu`, `cqv`, `cqw`, `alt`, and `php`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        omega: &'a mut Field,
        west_east_moisture: &'a mut Field,
        south_north_moisture: &'a mut Field,
        vertical_moisture: &'a mut Field,
        full_inverse_density: &'a mut Field,
        pressure_point_geopotential: &'a mut Field,
    ) -> Self {
        Self {
            omega,
            west_east_moisture,
            south_north_moisture,
            vertical_moisture,
            full_inverse_density,
            pressure_point_geopotential,
        }
    }
}
