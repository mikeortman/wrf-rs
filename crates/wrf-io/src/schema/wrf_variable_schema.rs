use crate::{
    WrfAttribute, WrfAttributeValue, WrfDataType, WrfDimensionName, WrfIoResult, WrfVariableName,
};

/// Typed schema for one WRF NetCDF variable.
#[derive(Clone, Debug, PartialEq)]
pub struct WrfVariableSchema {
    name: WrfVariableName,
    data_type: WrfDataType,
    dimensions: Vec<WrfDimensionName>,
    attributes: Vec<WrfAttribute>,
}

impl WrfVariableSchema {
    /// Creates a variable schema from validated typed parts.
    pub fn try_new(
        name: impl Into<String>,
        data_type: WrfDataType,
        dimensions: Vec<WrfDimensionName>,
        attributes: Vec<WrfAttribute>,
    ) -> WrfIoResult<Self> {
        Ok(Self {
            name: WrfVariableName::try_new(name)?,
            data_type,
            dimensions,
            attributes,
        })
    }

    pub(crate) fn arw_float(
        name: &'static str,
        dimensions: Vec<WrfDimensionName>,
        memory_order: &'static str,
        description: &'static str,
        units: &'static str,
        stagger: &'static str,
    ) -> WrfIoResult<Self> {
        Self::wrf_field(
            name,
            WrfDataType::Float32,
            104,
            dimensions,
            memory_order,
            description,
            units,
            stagger,
        )
    }

    /// Builds one field with WRF's pinned attribute set and order.
    ///
    /// The attribute order mirrors WRF's writer: `FieldType` and `MemoryOrder`
    /// come from `ext_ncd_write_field`, then `description`, `units`, and
    /// `stagger` follow from `wrf_ext_write_field` metadata.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn wrf_field(
        name: impl Into<String>,
        data_type: WrfDataType,
        field_type: i32,
        dimensions: Vec<WrfDimensionName>,
        memory_order: &str,
        description: &str,
        units: &str,
        stagger: &str,
    ) -> WrfIoResult<Self> {
        Self::try_new(
            name,
            data_type,
            dimensions,
            vec![
                WrfAttribute::new("FieldType", WrfAttributeValue::Int32(vec![field_type])),
                WrfAttribute::new(
                    "MemoryOrder",
                    WrfAttributeValue::Text(memory_order.to_owned()),
                ),
                WrfAttribute::new(
                    "description",
                    WrfAttributeValue::Text(description.to_owned()),
                ),
                WrfAttribute::new("units", WrfAttributeValue::Text(units.to_owned())),
                WrfAttribute::new("stagger", WrfAttributeValue::Text(stagger.to_owned())),
            ],
        )
    }

    /// Returns the validated variable name.
    pub const fn name(&self) -> &WrfVariableName {
        &self.name
    }

    /// Returns the file primitive type.
    pub const fn data_type(&self) -> WrfDataType {
        self.data_type
    }

    /// Returns file-order dimensions, including `Time` when present.
    pub fn dimensions(&self) -> &[WrfDimensionName] {
        &self.dimensions
    }

    /// Returns ordered variable attributes.
    pub fn attributes(&self) -> &[WrfAttribute] {
        &self.attributes
    }
}
