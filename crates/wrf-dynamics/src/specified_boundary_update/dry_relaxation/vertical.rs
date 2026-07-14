use wrf_compute::FieldStorage;

use super::DryBoundaryRelaxationBoundaryData;

/// Whether WRF's nested-only vertical-momentum relaxation is active.
pub enum DryBoundaryVerticalRelaxation<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Global specified domains do not relax vertical momentum here.
    Disabled,
    /// Nested domains mass-weight vertical velocity and relax its tendency.
    Nested {
        /// Uncoupled vertical velocity (`w`).
        velocity: &'a Field,
        /// Mutable coupled vertical-momentum tendency (`rw_tendf`).
        tendency: &'a mut Field,
        /// Boundary-file vertical-velocity values and tendencies.
        boundary: DryBoundaryRelaxationBoundaryData<'a, Field>,
    },
}
