#[derive(Clone, Copy)]
pub(crate) struct OmegaDiagnosisLevelCoefficients {
    pub(super) mass_multiplier: f32,
    pub(super) mass_offset: f32,
    pub(super) eta_layer_thickness: f32,
}

impl OmegaDiagnosisLevelCoefficients {
    pub(crate) const fn new(
        mass_multiplier: f32,
        mass_offset: f32,
        eta_layer_thickness: f32,
    ) -> Self {
        Self {
            mass_multiplier,
            mass_offset,
            eta_layer_thickness,
        }
    }
}
