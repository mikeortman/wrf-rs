/// Mutable mass-coupled momentum outputs.
pub struct MomentumCouplingOutputs<'a, Field> {
    pub(crate) west_east: &'a mut Field,
    pub(crate) south_north: &'a mut Field,
    pub(crate) vertical: &'a mut Field,
}

impl<'a, Field> MomentumCouplingOutputs<'a, Field> {
    /// Groups the `ru`, `rv`, and `rw` outputs without allocating or copying.
    pub fn new(
        west_east: &'a mut Field,
        south_north: &'a mut Field,
        vertical: &'a mut Field,
    ) -> Self {
        Self {
            west_east,
            south_north,
            vertical,
        }
    }
}

/// Immutable velocity fields before dry-air-mass coupling.
#[derive(Clone, Copy)]
pub struct MomentumCouplingVelocities<'a, Field> {
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) vertical: &'a Field,
}

impl<'a, Field> MomentumCouplingVelocities<'a, Field> {
    /// Groups the `u`, `v`, and `w` velocity fields without copying.
    pub const fn new(west_east: &'a Field, south_north: &'a Field, vertical: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
            vertical,
        }
    }
}

/// Immutable full column masses on the three relevant C-grid locations.
#[derive(Clone, Copy)]
pub struct MomentumCouplingMasses<'a, Field> {
    pub(crate) west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) mass_point: &'a Field,
}

impl<'a, Field> MomentumCouplingMasses<'a, Field> {
    /// Groups the `muu`, `muv`, and `mut` horizontal fields without copying.
    pub const fn new(west_east: &'a Field, south_north: &'a Field, mass_point: &'a Field) -> Self {
        Self {
            west_east,
            south_north,
            mass_point,
        }
    }
}

/// Immutable map-scale fields used by the three coupling expressions.
#[derive(Clone, Copy)]
pub struct MomentumCouplingMapFactors<'a, Field> {
    pub(crate) west_east: &'a Field,
    pub(crate) inverse_south_north: &'a Field,
    pub(crate) mass_point: &'a Field,
}

impl<'a, Field> MomentumCouplingMapFactors<'a, Field> {
    /// Groups `msfu`, `msfv_inv`, and `msft` without carrying WRF's unused
    /// non-inverse `msfv` argument into the Rust API.
    pub const fn new(
        west_east: &'a Field,
        inverse_south_north: &'a Field,
        mass_point: &'a Field,
    ) -> Self {
        Self {
            west_east,
            inverse_south_north,
            mass_point,
        }
    }
}
