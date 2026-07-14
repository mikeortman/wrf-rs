/// Mutable Runge–Kutta tendencies produced by dry-tendency assembly.
pub struct DryTendencyAssemblyRungeKuttaTendencies<'a, Field> {
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
    pub(crate) vertical_momentum: &'a mut Field,
    pub(crate) geopotential: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) column_mass: &'a mut Field,
}

impl<'a, Field> DryTendencyAssemblyRungeKuttaTendencies<'a, Field> {
    /// Groups the six mutable RK tendency fields without allocating.
    pub fn new(
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
        vertical_momentum: &'a mut Field,
        geopotential: &'a mut Field,
        potential_temperature: &'a mut Field,
        column_mass: &'a mut Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            geopotential,
            potential_temperature,
            column_mass,
        }
    }
}

/// Persistent physics/forward tendencies, updated only on the first substep.
pub struct DryTendencyAssemblyForwardTendencies<'a, Field> {
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
    pub(crate) vertical_momentum: &'a mut Field,
    pub(crate) geopotential: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
    pub(crate) column_mass: &'a Field,
}

impl<'a, Field> DryTendencyAssemblyForwardTendencies<'a, Field> {
    /// Groups WRF's `*tendf` fields; column mass is immutable in the routine.
    pub fn new(
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
        vertical_momentum: &'a mut Field,
        geopotential: &'a mut Field,
        potential_temperature: &'a mut Field,
        column_mass: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            geopotential,
            potential_temperature,
            column_mass,
        }
    }
}

/// Saved boundary-condition tendencies accumulated on the first substep.
#[derive(Clone, Copy)]
pub struct DryTendencyAssemblySavedTendencies<'a, Field> {
    pub(crate) west_east_momentum: &'a Field,
    pub(crate) south_north_momentum: &'a Field,
    pub(crate) vertical_momentum: &'a Field,
    pub(crate) geopotential: &'a Field,
    pub(crate) potential_temperature: &'a Field,
}

impl<'a, Field> DryTendencyAssemblySavedTendencies<'a, Field> {
    /// Groups WRF's five `*_save` fields without copying.
    pub const fn new(
        west_east_momentum: &'a Field,
        south_north_momentum: &'a Field,
        vertical_momentum: &'a Field,
        geopotential: &'a Field,
        potential_temperature: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            geopotential,
            potential_temperature,
        }
    }
}

/// Thermodynamic fields used by the potential-temperature equation.
#[derive(Clone, Copy)]
pub struct DryTendencyAssemblyThermodynamics<'a, Field> {
    pub(crate) diabatic_heating: &'a Field,
    pub(crate) full_column_mass: &'a Field,
}

impl<'a, Field> DryTendencyAssemblyThermodynamics<'a, Field> {
    /// Groups `h_diabatic` and `mut` without copying.
    pub const fn new(diabatic_heating: &'a Field, full_column_mass: &'a Field) -> Self {
        Self {
            diabatic_heating,
            full_column_mass,
        }
    }
}

/// The four map-factor fields actually read by WRF `rk_addtend_dry`.
#[derive(Clone, Copy)]
pub struct DryTendencyAssemblyMapFactors<'a, Field> {
    pub(crate) west_east_momentum_south_north: &'a Field,
    pub(crate) south_north_momentum_west_east: &'a Field,
    pub(crate) inverse_south_north_momentum_west_east: &'a Field,
    pub(crate) mass_point_south_north: &'a Field,
}

impl<'a, Field> DryTendencyAssemblyMapFactors<'a, Field> {
    /// Groups `msfuy`, `msfvx`, `msfvx_inv`, and `msfty`.
    pub const fn new(
        west_east_momentum_south_north: &'a Field,
        south_north_momentum_west_east: &'a Field,
        inverse_south_north_momentum_west_east: &'a Field,
        mass_point_south_north: &'a Field,
    ) -> Self {
        Self {
            west_east_momentum_south_north,
            south_north_momentum_west_east,
            inverse_south_north_momentum_west_east,
            mass_point_south_north,
        }
    }
}
