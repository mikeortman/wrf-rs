use wrf_compute::FieldStorage;

use crate::{
    AcousticVerticalGeopotentialInputs, AcousticVerticalLevelCoefficients,
    AcousticVerticalMapFactors, AcousticVerticalMassInputs, AcousticVerticalMomentumInputs,
    AcousticVerticalSolveInputs, AcousticVerticalThermodynamicInputs,
};

/// Complete role-grouped borrowed input set for `advance_w`.
#[derive(Debug)]
pub struct AcousticVerticalInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) momentum: AcousticVerticalMomentumInputs<'a, Field>,
    pub(crate) mass: AcousticVerticalMassInputs<'a, Field>,
    pub(crate) thermodynamics: AcousticVerticalThermodynamicInputs<'a, Field>,
    pub(crate) geopotential: AcousticVerticalGeopotentialInputs<'a, Field>,
    pub(crate) maps: AcousticVerticalMapFactors<'a, Field>,
    pub(crate) solve: AcousticVerticalSolveInputs<'a, Field>,
    pub(crate) vertical: AcousticVerticalLevelCoefficients<'a>,
}

impl<Field> Copy for AcousticVerticalInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups immutable input bundles without copying field data.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        momentum: AcousticVerticalMomentumInputs<'a, Field>,
        mass: AcousticVerticalMassInputs<'a, Field>,
        thermodynamics: AcousticVerticalThermodynamicInputs<'a, Field>,
        geopotential: AcousticVerticalGeopotentialInputs<'a, Field>,
        maps: AcousticVerticalMapFactors<'a, Field>,
        solve: AcousticVerticalSolveInputs<'a, Field>,
        vertical: AcousticVerticalLevelCoefficients<'a>,
    ) -> Self {
        Self {
            momentum,
            mass,
            thermodynamics,
            geopotential,
            maps,
            solve,
            vertical,
        }
    }
}
