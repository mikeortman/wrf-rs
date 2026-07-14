use wrf_compute::FieldStorage;

use crate::SpecifiedBoundaryTendencies;

/// Whether WRF's nested-only vertical-momentum tendency assignment is active.
pub enum DryBoundaryVerticalTendency<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Global specified domains leave vertical momentum unchanged here.
    Disabled,
    /// Nested domains assign vertical-momentum boundary-file tendencies.
    Nested {
        /// Mutable vertical-momentum tendency (`rw_tend`).
        tendency: &'a mut Field,
        /// Boundary-file W tendencies.
        boundaries: SpecifiedBoundaryTendencies<'a, Field>,
    },
}
