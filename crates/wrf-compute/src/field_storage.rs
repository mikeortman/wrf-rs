use crate::GridShape;

/// Backend-owned storage for one logically three-dimensional WRF field.
///
/// The trait intentionally exposes shape but not host slices. CPU-specific
/// kernels may use [`crate::CpuField`] slices, while a future GPU field can keep
/// values resident on its device.
pub trait FieldStorage<Value>: Send + Sync {
    /// Returns the logical shape shared by every backend representation.
    fn shape(&self) -> GridShape;

    /// Returns the number of scalar values in this field.
    fn value_count(&self) -> usize {
        self.shape().point_count()
    }
}
