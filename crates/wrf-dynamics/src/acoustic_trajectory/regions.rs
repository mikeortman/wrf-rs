use crate::{
    AcousticFluxAccumulationRegion, AcousticHorizontalMomentumRegion, AcousticMassThetaRegion,
    AcousticPressureRegion, AcousticStepPreparationRegion, AcousticVerticalRegion,
    VerticalAcousticCoefficientRegion,
};

/// Validated region set used by each stage of an acoustic trajectory.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryRegions<'a> {
    pub(crate) preparation: &'a AcousticStepPreparationRegion,
    pub(crate) pressure: &'a AcousticPressureRegion,
    pub(crate) vertical_coefficients: &'a VerticalAcousticCoefficientRegion,
    pub(crate) horizontal_momentum: &'a AcousticHorizontalMomentumRegion,
    pub(crate) mass_theta: &'a AcousticMassThetaRegion,
    pub(crate) vertical_momentum: &'a AcousticVerticalRegion,
    pub(crate) flux_accumulation: &'a AcousticFluxAccumulationRegion,
}

impl<'a> AcousticTrajectoryRegions<'a> {
    /// Groups the seven existing typed regions without recomputing ranges.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        preparation: &'a AcousticStepPreparationRegion,
        pressure: &'a AcousticPressureRegion,
        vertical_coefficients: &'a VerticalAcousticCoefficientRegion,
        horizontal_momentum: &'a AcousticHorizontalMomentumRegion,
        mass_theta: &'a AcousticMassThetaRegion,
        vertical_momentum: &'a AcousticVerticalRegion,
        flux_accumulation: &'a AcousticFluxAccumulationRegion,
    ) -> Self {
        Self {
            preparation,
            pressure,
            vertical_coefficients,
            horizontal_momentum,
            mass_theta,
            vertical_momentum,
            flux_accumulation,
        }
    }
}
