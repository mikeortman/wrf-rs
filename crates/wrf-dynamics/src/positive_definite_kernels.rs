use wrf_compute::FieldStorage;

use crate::{PositiveDefiniteResult, PositiveDefiniteSlabRegion};

/// Backend capability for WRF's positive-definite correction kernels.
///
/// The associated field type keeps CPU slices and future device-resident GPU
/// buffers behind the same numerical capability without pretending that an
/// arbitrary host closure is portable to a device.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::PositiveDefiniteKernels;
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(4, 1, 1)?;
/// let mut mixing_ratio = backend.create_field(shape, 0.0_f32)?;
/// mixing_ratio.values_mut().copy_from_slice(&[-1.0, 1.0, 2.0, 4.0]);
///
/// backend.apply_positive_definite_sheet(&mut mixing_ratio, &[10.0])?;
/// assert_eq!(mixing_ratio.values(), &[0.0, 2.0, 3.0, 5.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait PositiveDefiniteKernels {
    /// Native field storage used by this backend.
    type Field: FieldStorage<f32>;

    /// Applies WRF's `positive_definite_sheet` to contiguous west-east lines.
    ///
    /// Lines that already contain no negative value remain bit-for-bit
    /// unchanged. A corrected line is translated by its minimum and scaled to
    /// its supplied total. Degenerate lines and lines with negative totals are
    /// filled with zero, matching WRF v4.7.1.
    ///
    /// # Errors
    ///
    /// Returns an error when the field is not a two-dimensional sheet, the
    /// number of totals differs from the number of lines, or execution fails.
    fn apply_positive_definite_sheet(
        &self,
        field: &mut Self::Field,
        line_totals: &[f32],
    ) -> PositiveDefiniteResult<()>;

    /// Applies WRF's `positive_definite_slab` to an active three-dimensional region.
    ///
    /// Every selected west-east line derives its target total from its original
    /// values. Storage outside the region remains untouched.
    ///
    /// # Errors
    ///
    /// Returns an error when the region was constructed for a different field
    /// shape or execution fails.
    fn apply_positive_definite_slab(
        &self,
        field: &mut Self::Field,
        region: &PositiveDefiniteSlabRegion,
    ) -> PositiveDefiniteResult<()>;
}
