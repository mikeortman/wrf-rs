/// Field slices used by Kessler warm-rain conversion.
pub(crate) struct KesslerWarmRainFields<'a> {
    pub(crate) potential_temperature: &'a mut [f32],
    pub(crate) water_vapor_mixing_ratio: &'a mut [f32],
    pub(crate) cloud_water_mixing_ratio: &'a mut [f32],
    pub(crate) rain_water_mixing_ratio: &'a mut [f32],
    pub(crate) dry_air_density: &'a [f32],
    pub(crate) exner_function: &'a [f32],
    pub(crate) production: &'a [f32],
}
