/// Sentinel-based definition-table value generated as WRF's `PARAM_*` constant.
///
/// WRF emits zero for the reserved placeholder and one for the first real
/// member even though the generated Fortran table itself is one-based. This
/// value is therefore not interchangeable with a Fortran array subscript.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DefinitionParameterIndex(usize);

impl DefinitionParameterIndex {
    pub(crate) const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the emitted `PARAM_*` value (`0` is the reserved sentinel).
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_the_definition_table_position() {
        assert_eq!(DefinitionParameterIndex::new(3).as_usize(), 3);
    }
}
