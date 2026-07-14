use wrf_compute::FieldStorage;

/// Borrowed fields and vertical coefficients for a geopotential boundary update.
#[derive(Clone, Copy)]
pub struct SpecifiedBoundaryGeopotentialInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) saved_geopotential: &'a Field,
    pub(crate) geopotential_tendency: &'a Field,
    pub(crate) column_mass_tendency: &'a Field,
    pub(crate) current_column_mass: &'a Field,
    pub(crate) column_mass_multiplier: &'a [f32],
    pub(crate) column_mass_offset: &'a [f32],
}

impl<'a, Field> SpecifiedBoundaryGeopotentialInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the immutable inputs used by WRF `spec_bdyupdate_ph`.
    pub const fn new(
        saved_geopotential: &'a Field,
        geopotential_tendency: &'a Field,
        column_mass_tendency: &'a Field,
        current_column_mass: &'a Field,
        column_mass_multiplier: &'a [f32],
        column_mass_offset: &'a [f32],
    ) -> Self {
        Self {
            saved_geopotential,
            geopotential_tendency,
            column_mass_tendency,
            current_column_mass,
            column_mass_multiplier,
            column_mass_offset,
        }
    }
}
