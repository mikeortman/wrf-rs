/// Borrowed fields participating in one microphysics driver update.
///
/// Moisture species fields are supplied as one slice ordered exactly like the
/// scheme's [`crate::MoistureSpeciesPackage`], mirroring WRF's Registry-packed
/// four-dimensional `moist` state. All borrows are views; no field is cloned.
pub struct MicrophysicsDriverFields<'a, Field> {
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) moisture_species_fields: &'a mut [Field],
    pub(crate) dry_air_density: &'a Field,
    pub(crate) exner_function: &'a Field,
    pub(crate) height: &'a Field,
    pub(crate) vertical_layer_thickness: &'a Field,
    pub(crate) accumulated_precipitation: &'a mut Field,
    pub(crate) step_precipitation: &'a mut Field,
}

impl<'a, Field> MicrophysicsDriverFields<'a, Field> {
    /// Groups the complete driver field set without allocating or copying.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        potential_temperature: &'a mut Field,
        moisture_species_fields: &'a mut [Field],
        dry_air_density: &'a Field,
        exner_function: &'a Field,
        height: &'a Field,
        vertical_layer_thickness: &'a Field,
        accumulated_precipitation: &'a mut Field,
        step_precipitation: &'a mut Field,
    ) -> Self {
        Self {
            potential_temperature,
            moisture_species_fields,
            dry_air_density,
            exner_function,
            height,
            vertical_layer_thickness,
            accumulated_precipitation,
            step_precipitation,
        }
    }
}
