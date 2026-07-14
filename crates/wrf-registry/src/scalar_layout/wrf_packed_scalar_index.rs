/// One-based packed scalar position generated as WRF's runtime `P_*` value.
///
/// Position one is permanently reserved, so the first active scalar receives
/// packed index two.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WrfPackedScalarIndex(usize);

impl WrfPackedScalarIndex {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the one-based packed WRF position.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_the_one_based_packed_position() {
        assert_eq!(WrfPackedScalarIndex::new(2).as_usize(), 2);
    }
}
