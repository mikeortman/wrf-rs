use crate::{
    ColumnMassStaggeringRegion, InverseDensityRegion, MoistureCoefficientRegion,
    MomentumCouplingRegion, OmegaDiagnosisRegion, PressurePointGeopotentialRegion,
};

/// Validated regions for all seven Runge-Kutta preparation stages.
///
/// The component regions remain distinct because WRF's stagger clipping and
/// vertical-neighbor contracts differ by diagnostic. They are small metadata
/// values, so this bundle owns them and is cheap to clone when a caller wants
/// to reuse a setup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RungeKuttaPreparationRegions {
    pub(crate) column_mass: ColumnMassStaggeringRegion,
    pub(crate) momentum: MomentumCouplingRegion,
    pub(crate) omega: OmegaDiagnosisRegion,
    pub(crate) moisture: MoistureCoefficientRegion,
    pub(crate) inverse_density: InverseDensityRegion,
    pub(crate) pressure_point_geopotential: PressurePointGeopotentialRegion,
}

impl RungeKuttaPreparationRegions {
    /// Groups independently validated component regions without weakening them.
    pub const fn new(
        column_mass: ColumnMassStaggeringRegion,
        momentum: MomentumCouplingRegion,
        omega: OmegaDiagnosisRegion,
        moisture: MoistureCoefficientRegion,
        inverse_density: InverseDensityRegion,
        pressure_point_geopotential: PressurePointGeopotentialRegion,
    ) -> Self {
        Self {
            column_mass,
            momentum,
            omega,
            moisture,
            inverse_density,
            pressure_point_geopotential,
        }
    }
}
