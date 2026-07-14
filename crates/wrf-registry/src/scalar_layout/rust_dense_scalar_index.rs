/// Zero-based dense Rust position for one active scalar-array member.
///
/// This index deliberately excludes WRF's reserved packed slot and is the
/// position used by Rust field slices.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RustDenseScalarIndex(usize);

impl RustDenseScalarIndex {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based dense Rust position.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_the_zero_based_dense_position() {
        assert_eq!(RustDenseScalarIndex::new(0).as_usize(), 0);
    }
}
