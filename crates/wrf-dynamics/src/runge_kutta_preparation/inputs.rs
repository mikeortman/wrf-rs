use crate::{MoistureSpecies, OmegaDiagnosisGridMetrics};

use super::{
    RungeKuttaPreparationCoefficients, RungeKuttaPreparationMapFactors,
    RungeKuttaPreparationMassInputs, RungeKuttaPreparationThermodynamicInputs,
    RungeKuttaPreparationVelocities,
};

/// Complete immutable input set for one Runge-Kutta preparation pass.
pub struct RungeKuttaPreparationInputs<'a, Field> {
    pub(crate) masses: RungeKuttaPreparationMassInputs<'a, Field>,
    pub(crate) velocities: RungeKuttaPreparationVelocities<'a, Field>,
    pub(crate) map_factors: RungeKuttaPreparationMapFactors<'a, Field>,
    pub(crate) coefficients: RungeKuttaPreparationCoefficients<'a>,
    pub(crate) moisture: MoistureSpecies<'a, Field>,
    pub(crate) thermodynamics: RungeKuttaPreparationThermodynamicInputs<'a, Field>,
    pub(crate) grid_metrics: OmegaDiagnosisGridMetrics,
}

impl<'a, Field> RungeKuttaPreparationInputs<'a, Field> {
    /// Builds the borrowed input view without allocating or cloning fields.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        masses: RungeKuttaPreparationMassInputs<'a, Field>,
        velocities: RungeKuttaPreparationVelocities<'a, Field>,
        map_factors: RungeKuttaPreparationMapFactors<'a, Field>,
        coefficients: RungeKuttaPreparationCoefficients<'a>,
        moisture: MoistureSpecies<'a, Field>,
        thermodynamics: RungeKuttaPreparationThermodynamicInputs<'a, Field>,
        grid_metrics: OmegaDiagnosisGridMetrics,
    ) -> Self {
        Self {
            masses,
            velocities,
            map_factors,
            coefficients,
            moisture,
            thermodynamics,
            grid_metrics,
        }
    }
}
