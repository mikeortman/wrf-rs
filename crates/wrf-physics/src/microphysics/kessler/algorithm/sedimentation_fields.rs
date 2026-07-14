/// Field slices used by the Kessler sedimentation phase.
pub(crate) struct KesslerSedimentationFields<'a> {
    pub(crate) rain_water_mixing_ratio: &'a [f32],
    pub(crate) dry_air_density: &'a [f32],
    pub(crate) height: &'a [f32],
    pub(crate) vertical_layer_thickness: &'a [f32],
    pub(crate) production: &'a mut [f32],
    pub(crate) accumulated_precipitation: &'a mut [f32],
    pub(crate) step_precipitation: &'a mut [f32],
}
