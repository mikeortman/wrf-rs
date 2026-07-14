use wrf_compute::FieldStorage;

/// Mutable tridiagonal state produced for the vertical acoustic solve.
#[derive(Debug)]
pub struct VerticalAcousticSolveCoefficients<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) lower_diagonal: &'a mut Field,
    pub(crate) inverse_eliminated_diagonal: &'a mut Field,
    pub(crate) upper_elimination_factor: &'a mut Field,
}

impl<'a, Field> VerticalAcousticSolveCoefficients<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `a`, `alpha`, and `gamma` as distinct mutable roles.
    pub const fn new(
        lower_diagonal: &'a mut Field,
        inverse_eliminated_diagonal: &'a mut Field,
        upper_elimination_factor: &'a mut Field,
    ) -> Self {
        Self {
            lower_diagonal,
            inverse_eliminated_diagonal,
            upper_elimination_factor,
        }
    }
}
