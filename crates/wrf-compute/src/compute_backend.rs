use crate::{BackendKind, ComputeResult, FieldStorage, FieldValue, GridShape};

/// Owns field allocation for one physical compute backend.
///
/// Numerical crates should add narrow kernel capability traits instead of
/// expanding this trait into a universal collection of unrelated operations.
pub trait ComputeBackend: Send + Sync {
    /// Field representation owned by this backend.
    type Field<Value>: FieldStorage<Value>
    where
        Value: FieldValue;

    /// Returns the processor family used by this backend.
    fn backend_kind(&self) -> BackendKind;

    /// Allocates a field initialized to the same scalar value everywhere.
    fn create_field<Value>(
        &self,
        shape: GridShape,
        initial_value: Value,
    ) -> ComputeResult<Self::Field<Value>>
    where
        Value: FieldValue;
}
