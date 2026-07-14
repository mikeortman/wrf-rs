/// Borrowed fields participating in one Kessler microphysics update.
///
/// Four prognostic fields and two precipitation fields are updated in place.
/// Immutable thermodynamic and geometric inputs remain shared across workers;
/// no domain-sized field is cloned.
pub struct KesslerMicrophysicsFields<'a, Field> {
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) water_vapor_mixing_ratio: &'a mut Field,
    pub(crate) cloud_water_mixing_ratio: &'a mut Field,
    pub(crate) rain_water_mixing_ratio: &'a mut Field,
    pub(crate) dry_air_density: &'a Field,
    pub(crate) exner_function: &'a Field,
    pub(crate) height: &'a Field,
    pub(crate) vertical_layer_thickness: &'a Field,
    pub(crate) accumulated_precipitation: &'a mut Field,
    pub(crate) step_precipitation: &'a mut Field,
}

impl<'a, Field> KesslerMicrophysicsFields<'a, Field> {
    /// Groups the complete Kessler field set without allocating or copying.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        potential_temperature: &'a mut Field,
        water_vapor_mixing_ratio: &'a mut Field,
        cloud_water_mixing_ratio: &'a mut Field,
        rain_water_mixing_ratio: &'a mut Field,
        dry_air_density: &'a Field,
        exner_function: &'a Field,
        height: &'a Field,
        vertical_layer_thickness: &'a Field,
        accumulated_precipitation: &'a mut Field,
        step_precipitation: &'a mut Field,
    ) -> Self {
        Self {
            potential_temperature,
            water_vapor_mixing_ratio,
            cloud_water_mixing_ratio,
            rain_water_mixing_ratio,
            dry_air_density,
            exner_function,
            height,
            vertical_layer_thickness,
            accumulated_precipitation,
            step_precipitation,
        }
    }
}
