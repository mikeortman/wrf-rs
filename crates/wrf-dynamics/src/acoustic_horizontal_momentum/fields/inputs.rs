use wrf_compute::FieldStorage;

use crate::{
    AcousticHorizontalMapFactors, AcousticHorizontalMassFields,
    AcousticHorizontalMoistureCoefficients, AcousticHorizontalMomentumTendencies,
    AcousticHorizontalPressureFields, AcousticHorizontalVerticalCoefficients,
};

/// Complete role-grouped input set for acoustic horizontal-momentum advancement.
#[derive(Debug)]
pub struct AcousticHorizontalMomentumInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) tendencies: AcousticHorizontalMomentumTendencies<'a, Field>,
    pub(crate) pressure: AcousticHorizontalPressureFields<'a, Field>,
    pub(crate) masses: AcousticHorizontalMassFields<'a, Field>,
    pub(crate) moisture: AcousticHorizontalMoistureCoefficients<'a, Field>,
    pub(crate) map_factors: AcousticHorizontalMapFactors<'a, Field>,
    pub(crate) vertical: AcousticHorizontalVerticalCoefficients<'a>,
}

impl<Field> Copy for AcousticHorizontalMomentumInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticHorizontalMomentumInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticHorizontalMomentumInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the role-specific borrowed input bundles without copying fields.
    pub const fn new(
        tendencies: AcousticHorizontalMomentumTendencies<'a, Field>,
        pressure: AcousticHorizontalPressureFields<'a, Field>,
        masses: AcousticHorizontalMassFields<'a, Field>,
        moisture: AcousticHorizontalMoistureCoefficients<'a, Field>,
        map_factors: AcousticHorizontalMapFactors<'a, Field>,
        vertical: AcousticHorizontalVerticalCoefficients<'a>,
    ) -> Self {
        Self {
            tendencies,
            pressure,
            masses,
            moisture,
            map_factors,
            vertical,
        }
    }
}
