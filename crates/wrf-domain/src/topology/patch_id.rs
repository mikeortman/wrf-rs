/// Stable row-major identifier for a process-grid patch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PatchId(usize);

impl PatchId {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the zero-based row-major identifier.
    pub const fn value(self) -> usize {
        self.0
    }
}
