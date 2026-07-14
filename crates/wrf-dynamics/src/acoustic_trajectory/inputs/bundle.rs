use wrf_compute::FieldStorage;

use crate::{
    AcousticTrajectoryMapFactors, AcousticTrajectoryMassInputs,
    AcousticTrajectoryMoistureCoefficients, AcousticTrajectoryPressureInputs,
    AcousticTrajectoryTendencies,
};

/// Complete immutable input set for a local acoustic trajectory.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryInputs<'a, Field: FieldStorage<f32>> {
    pub(crate) masses: AcousticTrajectoryMassInputs<'a, Field>,
    pub(crate) pressure: AcousticTrajectoryPressureInputs<'a, Field>,
    pub(crate) tendencies: AcousticTrajectoryTendencies<'a, Field>,
    pub(crate) moisture: AcousticTrajectoryMoistureCoefficients<'a, Field>,
    pub(crate) map_factors: AcousticTrajectoryMapFactors<'a, Field>,
}

impl<'a, Field: FieldStorage<f32>> AcousticTrajectoryInputs<'a, Field> {
    /// Groups immutable role-specific descriptors without copying fields.
    pub const fn new(
        masses: AcousticTrajectoryMassInputs<'a, Field>,
        pressure: AcousticTrajectoryPressureInputs<'a, Field>,
        tendencies: AcousticTrajectoryTendencies<'a, Field>,
        moisture: AcousticTrajectoryMoistureCoefficients<'a, Field>,
        map_factors: AcousticTrajectoryMapFactors<'a, Field>,
    ) -> Self {
        Self {
            masses,
            pressure,
            tendencies,
            moisture,
            map_factors,
        }
    }
}
