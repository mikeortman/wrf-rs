use wrf_compute::FieldStorage;

use crate::{SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryTendencies};

use super::DryLargeStepRelaxationInputs;

/// Nested-domain vertical-momentum data for the first Runge–Kutta substep.
///
/// The boundary tendencies feed both W relaxation and W boundary assignment;
/// the velocity and boundary values feed only relaxation.
pub struct DryLargeStepNestedVertical<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) velocity: &'a Field,
    pub(crate) boundary_values: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
    pub(crate) boundary_tendencies: SpecifiedBoundaryTendencies<'a, Field>,
}

impl<'a, Field> DryLargeStepNestedVertical<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups the uncoupled velocity (`w`) with its W boundary-file arrays.
    pub const fn new(
        velocity: &'a Field,
        boundary_values: SpecifiedBoundaryRelaxationBoundaryValues<'a, Field>,
        boundary_tendencies: SpecifiedBoundaryTendencies<'a, Field>,
    ) -> Self {
        Self {
            velocity,
            boundary_values,
            boundary_tendencies,
        }
    }
}

/// The four WRF execution modes of the dry large-step boundary stage.
///
/// The Runge–Kutta substep decides whether relaxation runs, and the domain
/// kind decides whether vertical momentum participates. Each variant carries
/// exactly the inputs its stages read, so no mode accepts unused data and no
/// inconsistent substep/domain combination is representable.
pub enum DryLargeStepBoundaryStageMode<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// `rk_step == 1` on a global specified domain: horizontal relaxation only.
    FirstSubstepGlobal {
        /// Inputs consumed only by `relax_bdy_dry`.
        relaxation: DryLargeStepRelaxationInputs<'a, Field>,
    },
    /// `rk_step == 1` on a nested domain: vertical momentum joins relaxation
    /// and boundary assignment.
    FirstSubstepNested {
        /// Inputs consumed only by `relax_bdy_dry`.
        relaxation: DryLargeStepRelaxationInputs<'a, Field>,
        /// Nested W state and boundary-file arrays.
        vertical: DryLargeStepNestedVertical<'a, Field>,
    },
    /// A later substep on a global specified domain skips relaxation.
    LaterSubstepGlobal,
    /// A later substep on a nested domain skips relaxation but still assigns
    /// W boundary-file tendencies.
    LaterSubstepNested {
        /// Four W tendency boundary slabs in west, east, south, north order.
        vertical_boundaries: SpecifiedBoundaryTendencies<'a, Field>,
    },
}
