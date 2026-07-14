use wrf_compute::FieldStorage;

use crate::SpecifiedBoundaryTendencies;

/// Boundary policy for vertical momentum after dry-tendency assembly.
pub enum DryTendencyBoundaryStageVertical<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// A global specified domain assembles W but does not overwrite its edge.
    Global,
    /// A nested domain assigns W from its boundary-file tendencies.
    Nested {
        /// Four W tendency boundary slabs in west, east, south, north order.
        boundaries: SpecifiedBoundaryTendencies<'a, Field>,
    },
}
