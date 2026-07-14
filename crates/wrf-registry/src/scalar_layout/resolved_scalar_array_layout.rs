use crate::{
    DefinitionParameterIndex, ResolvedScalarArrayMember, SourceLocation, WrfPackedScalarIndex,
};

/// Active packed layout for one four-dimensional Registry scalar array.
///
/// Members are ordered by selected package source order, then group/member
/// order. Repeated activation reuses the first member and does not append a
/// duplicate, matching WRF's generated index table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedScalarArrayLayout {
    pub(crate) location: SourceLocation,
    pub(crate) scalar_array_name: String,
    pub(crate) definition_member_count: usize,
    pub(crate) members: Vec<ResolvedScalarArrayMember>,
}

impl ResolvedScalarArrayLayout {
    /// Returns the physical location of the array's reserved `state` definition.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the four-dimensional scalar-array name.
    #[must_use]
    pub fn scalar_array_name(&self) -> &str {
        &self.scalar_array_name
    }

    /// Returns the number of definition-table members, including the reserved slot.
    #[must_use]
    pub const fn definition_member_count(&self) -> usize {
        self.definition_member_count
    }

    /// Returns the reserved definition-table index (`PARAM` zero).
    #[must_use]
    pub const fn reserved_parameter_index(&self) -> DefinitionParameterIndex {
        DefinitionParameterIndex::new(0)
    }

    /// Returns the reserved one-based WRF packed position (`P` one).
    #[must_use]
    pub const fn reserved_packed_scalar_index(&self) -> WrfPackedScalarIndex {
        WrfPackedScalarIndex::new(1)
    }

    /// Returns active members in their resolved packed order.
    #[must_use]
    pub fn members(&self) -> &[ResolvedScalarArrayMember] {
        &self.members
    }
}
