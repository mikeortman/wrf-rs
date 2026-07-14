use wrf_compute::FieldStorage;

use crate::{
    AcousticMassThetaMapFactors, AcousticMassThetaMassInputs, AcousticMassThetaMomentumInputs,
    AcousticMassThetaThermodynamicInputs, AcousticMassThetaVerticalCoefficients,
};

/// Complete role-grouped borrowed input set for `advance_mu_t`.
#[derive(Debug)]
pub struct AcousticMassThetaInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) momentum: AcousticMassThetaMomentumInputs<'a, Field>,
    pub(crate) mass: AcousticMassThetaMassInputs<'a, Field>,
    pub(crate) thermodynamics: AcousticMassThetaThermodynamicInputs<'a, Field>,
    pub(crate) map_factors: AcousticMassThetaMapFactors<'a, Field>,
    pub(crate) vertical: AcousticMassThetaVerticalCoefficients<'a>,
}

impl<Field> Copy for AcousticMassThetaInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticMassThetaInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticMassThetaInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups all immutable input bundles without copying field data.
    pub const fn new(
        momentum: AcousticMassThetaMomentumInputs<'a, Field>,
        mass: AcousticMassThetaMassInputs<'a, Field>,
        thermodynamics: AcousticMassThetaThermodynamicInputs<'a, Field>,
        map_factors: AcousticMassThetaMapFactors<'a, Field>,
        vertical: AcousticMassThetaVerticalCoefficients<'a>,
    ) -> Self {
        Self {
            momentum,
            mass,
            thermodynamics,
            map_factors,
            vertical,
        }
    }
}
