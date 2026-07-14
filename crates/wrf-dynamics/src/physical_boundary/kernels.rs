use wrf_compute::FieldStorage;

use crate::{
    PhysicalBoundaryConditions, PhysicalBoundaryRegion, PhysicalBoundaryResult,
    PhysicalBoundaryVariable,
};

/// Backend capability for WRF physical boundary-zone assignment.
///
/// Implementations reproduce WRF v4.7.1 `set_physical_bc3d` and
/// `set_physical_bc2d` for single-rank patches: periodic wraps, symmetric
/// reflections, and open/specified/nested edge copies into halo storage.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     PhysicalBoundaryConditions, PhysicalBoundaryKernels, PhysicalBoundaryRegion,
///     PhysicalBoundaryVariable,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(15, 15, 7)?;
/// let mut field = backend.create_field(shape, 1.0_f32)?;
/// let region = PhysicalBoundaryRegion::try_new(
///     shape, 4..10, 4..10, 1..6, 4..11, 4..11, 1..7,
/// )?;
/// backend.apply_physical_boundary(
///     &mut field,
///     PhysicalBoundaryVariable::MassHalfLevel,
///     PhysicalBoundaryConditions::periodic_xy(),
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait PhysicalBoundaryKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Assigns the volume field's boundary zone (`set_physical_bc3d`).
    ///
    /// Branches fire only for tiles touching the corresponding domain edge,
    /// exactly as WRF tests `its == ids` and its analogues, so interior tiles
    /// are exact no-ops. Validation completes before mutation.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the field shape differs from `region`.
    fn apply_physical_boundary(
        &self,
        field: &mut Self::Field,
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
    ) -> PhysicalBoundaryResult<()>;

    /// Assigns the horizontal field's boundary zone (`set_physical_bc2d`).
    ///
    /// The field must use the region's horizontal storage shape. The
    /// two-dimensional routine keeps WRF's own quirks: its staggered
    /// symmetric ranges start at the edge point, and doubly periodic
    /// configurations additionally fill the four halo corners.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the field shape differs from the region's
    /// horizontal shape.
    fn apply_physical_boundary_horizontal(
        &self,
        field: &mut Self::Field,
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
    ) -> PhysicalBoundaryResult<()>;
}
