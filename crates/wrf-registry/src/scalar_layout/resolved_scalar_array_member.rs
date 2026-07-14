use crate::{DefinitionParameterIndex, RustDenseScalarIndex, SourceLocation, WrfPackedScalarIndex};

/// One active scalar-array member with all three non-interchangeable indices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedScalarArrayMember {
    pub(crate) location: SourceLocation,
    pub(crate) name: String,
    pub(crate) definition_parameter_index: DefinitionParameterIndex,
    pub(crate) wrf_packed_scalar_index: WrfPackedScalarIndex,
    pub(crate) rust_dense_scalar_index: RustDenseScalarIndex,
}

impl ResolvedScalarArrayMember {
    /// Returns the physical location of the member's `state` definition.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the Registry state/member name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the definition-table `PARAM_*` index, including the reserved member.
    #[must_use]
    pub const fn definition_parameter_index(&self) -> DefinitionParameterIndex {
        self.definition_parameter_index
    }

    /// Returns the one-based runtime WRF `P_*` position.
    #[must_use]
    pub const fn wrf_packed_scalar_index(&self) -> WrfPackedScalarIndex {
        self.wrf_packed_scalar_index
    }

    /// Returns the zero-based dense Rust field-slice position.
    #[must_use]
    pub const fn rust_dense_scalar_index(&self) -> RustDenseScalarIndex {
        self.rust_dense_scalar_index
    }
}
