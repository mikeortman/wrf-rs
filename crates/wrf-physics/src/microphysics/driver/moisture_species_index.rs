/// Zero-based position of one moisture species inside the ordered `moist` set.
///
/// WRF's Registry generates one-based Fortran constants such as `P_QV`; this
/// index is the zero-based Rust equivalent used to select the species field
/// from the package-ordered field slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MoistureSpeciesIndex(usize);

impl MoistureSpeciesIndex {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based species position.
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_wrapped_position() {
        assert_eq!(MoistureSpeciesIndex::new(2).as_usize(), 2);
    }
}
