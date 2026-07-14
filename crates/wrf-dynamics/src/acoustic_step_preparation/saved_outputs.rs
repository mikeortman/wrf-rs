/// Saved full state plus thermodynamic and omega preparation outputs.
pub struct AcousticStepPreparationSavedOutputs<'a, Field> {
    pub(crate) west_east_velocity: &'a mut Field,
    pub(crate) south_north_velocity: &'a mut Field,
    pub(crate) vertical_velocity: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) geopotential: &'a mut Field,
    pub(crate) column_mass: &'a mut Field,
    pub(crate) omega: &'a mut Field,
    pub(crate) pressure_coefficient: &'a mut Field,
}

impl<'a, Field> AcousticStepPreparationSavedOutputs<'a, Field> {
    /// Groups the eight saved/diagnostic outputs without allocation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        west_east_velocity: &'a mut Field,
        south_north_velocity: &'a mut Field,
        vertical_velocity: &'a mut Field,
        potential_temperature: &'a mut Field,
        geopotential: &'a mut Field,
        column_mass: &'a mut Field,
        omega: &'a mut Field,
        pressure_coefficient: &'a mut Field,
    ) -> Self {
        Self {
            west_east_velocity,
            south_north_velocity,
            vertical_velocity,
            potential_temperature,
            geopotential,
            column_mass,
            omega,
            pressure_coefficient,
        }
    }
}
