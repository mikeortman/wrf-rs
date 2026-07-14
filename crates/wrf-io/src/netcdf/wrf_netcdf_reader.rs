use std::path::{Path, PathBuf};

use netcdf::types::{FloatType, IntType, NcVariableType};
use netcdf::{AttributeValue, File};

use crate::{
    WrfAttribute, WrfAttributeValue, WrfDataType, WrfDimension, WrfDimensionName, WrfFileKind,
    WrfFileSchema, WrfIoError, WrfIoResult, WrfVariableName, WrfVariableSchema,
};

/// Reads typed WRF schema and fields through the thread-safe GeoRust NetCDF API.
///
/// The underlying library serializes NetCDF-C calls because NetCDF-C itself is
/// not thread-safe. Field data is read directly into caller-provided storage;
/// CPU model kernels remain independently multithreaded.
#[derive(Debug)]
pub struct WrfNetcdfReader {
    path: PathBuf,
    file: File,
    schema: WrfFileSchema,
}

impl WrfNetcdfReader {
    /// Opens a NetCDF-3 or NetCDF-4 file and inventories its supported WRF schema.
    pub fn open(path: impl AsRef<Path>) -> WrfIoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = netcdf::open(&path).map_err(|source| WrfIoError::NetcdfRead {
            path: path.clone(),
            variable: None,
            source,
        })?;
        let schema = Self::read_schema(&file, &path)?;
        Ok(Self { path, file, schema })
    }

    /// Returns the typed schema discovered in the file.
    pub const fn schema(&self) -> &WrfFileSchema {
        &self.schema
    }

    /// Reads a complete `NC_CHAR` variable into caller-owned storage.
    pub fn read_character_into(
        &self,
        name: &WrfVariableName,
        output: &mut [u8],
    ) -> WrfIoResult<()> {
        self.validate_output(name, WrfDataType::Character, output.len())?;
        let variable = self.find_variable(name)?;
        variable
            .get_raw_values_into(output, ..)
            .map_err(|source| self.read_error(name, source))
    }

    /// Reads a complete signed 32-bit variable into caller-owned storage.
    pub fn read_i32_into(&self, name: &WrfVariableName, output: &mut [i32]) -> WrfIoResult<()> {
        self.read_typed_into(name, WrfDataType::Int32, output)
    }

    /// Reads a complete single-precision variable into caller-owned storage.
    pub fn read_f32_into(&self, name: &WrfVariableName, output: &mut [f32]) -> WrfIoResult<()> {
        self.read_typed_into(name, WrfDataType::Float32, output)
    }

    /// Reads a complete double-precision variable into caller-owned storage.
    pub fn read_f64_into(&self, name: &WrfVariableName, output: &mut [f64]) -> WrfIoResult<()> {
        self.read_typed_into(name, WrfDataType::Float64, output)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn read_raw_chunk(
        &self,
        name: &WrfVariableName,
        start: &[usize],
        count: &[usize],
        output: &mut [u8],
    ) -> WrfIoResult<()> {
        self.find_variable(name)?
            .get_raw_values_into(output, (start, count))
            .map_err(|source| self.read_error(name, source))
    }

    fn read_typed_into<T>(
        &self,
        name: &WrfVariableName,
        expected_type: WrfDataType,
        output: &mut [T],
    ) -> WrfIoResult<()>
    where
        T: netcdf::NcTypeDescriptor + Copy,
    {
        self.validate_output(name, expected_type, output.len())?;
        self.find_variable(name)?
            .get_values_into(output, ..)
            .map_err(|source| self.read_error(name, source))
    }

    fn validate_output(
        &self,
        name: &WrfVariableName,
        expected_type: WrfDataType,
        actual_length: usize,
    ) -> WrfIoResult<()> {
        let variable = self
            .schema
            .variable(name)
            .ok_or_else(|| WrfIoError::VariableNotFound {
                variable: name.clone(),
            })?;
        if variable.data_type() != expected_type {
            return Err(WrfIoError::VariableTypeMismatch {
                variable: name.clone(),
                expected: variable.data_type(),
                actual: expected_type,
            });
        }

        let expected_length = self.schema.variable_element_count(variable)?;
        if expected_length != actual_length {
            return Err(WrfIoError::VariableLengthMismatch {
                variable: name.clone(),
                expected: expected_length,
                actual: actual_length,
            });
        }
        Ok(())
    }

    fn find_variable(&self, name: &WrfVariableName) -> WrfIoResult<netcdf::Variable<'_>> {
        self.file
            .variable(name.as_str())
            .ok_or_else(|| WrfIoError::VariableNotFound {
                variable: name.clone(),
            })
    }

    fn read_error(&self, name: &WrfVariableName, source: netcdf::Error) -> WrfIoError {
        WrfIoError::NetcdfRead {
            path: self.path.clone(),
            variable: Some(name.clone()),
            source,
        }
    }

    fn read_schema(file: &File, path: &Path) -> WrfIoResult<WrfFileSchema> {
        let dimensions = file
            .dimensions()
            .map(|dimension| {
                let name = WrfDimensionName::try_from_name(&dimension.name())?;
                Ok(if dimension.is_unlimited() {
                    WrfDimension::unlimited(name, dimension.len())
                } else {
                    WrfDimension::fixed(name, dimension.len())
                })
            })
            .collect::<WrfIoResult<Vec<_>>>()?;

        let attributes = file
            .attributes()
            .map(|attribute| Self::read_attribute(attribute, path))
            .collect::<WrfIoResult<Vec<_>>>()?;
        let file_kind = if attributes.iter().any(|attribute| {
            attribute.name() == "FLAG_RESTART"
                && attribute.value() == &WrfAttributeValue::Int32(vec![1])
        }) {
            WrfFileKind::Restart
        } else {
            WrfFileKind::Initialization
        };

        let variables = file
            .variables()
            .map(|variable| {
                let name = variable.name();
                let data_type = Self::map_data_type(&name, variable.vartype())?;
                let dimension_names = variable
                    .dimensions()
                    .iter()
                    .map(|dimension| WrfDimensionName::try_from_name(&dimension.name()))
                    .collect::<WrfIoResult<Vec<_>>>()?;
                let variable_attributes = variable
                    .attributes()
                    .map(|attribute| Self::read_attribute(attribute, path))
                    .collect::<WrfIoResult<Vec<_>>>()?;
                WrfVariableSchema::try_new(name, data_type, dimension_names, variable_attributes)
            })
            .collect::<WrfIoResult<Vec<_>>>()?;

        WrfFileSchema::try_from_parts(file_kind, dimensions, attributes, variables)
    }

    fn read_attribute(attribute: netcdf::Attribute<'_>, path: &Path) -> WrfIoResult<WrfAttribute> {
        let name = attribute.name().to_owned();
        let value = attribute.value().map_err(|source| WrfIoError::NetcdfRead {
            path: path.to_path_buf(),
            variable: None,
            source,
        })?;
        let value = match value {
            AttributeValue::Int(value) => WrfAttributeValue::Int32(vec![value]),
            AttributeValue::Ints(values) => WrfAttributeValue::Int32(values),
            AttributeValue::Float(value) => WrfAttributeValue::Float32(vec![value]),
            AttributeValue::Floats(values) => WrfAttributeValue::Float32(values),
            AttributeValue::Double(value) => WrfAttributeValue::Float64(vec![value]),
            AttributeValue::Doubles(values) => WrfAttributeValue::Float64(values),
            AttributeValue::Str(value) => {
                // Normalize WRF's one-NUL representation back to the logical
                // empty fixed-length character value used by the schema model.
                WrfAttributeValue::Text(if value == "\0" { String::new() } else { value })
            }
            actual => {
                return Err(WrfIoError::UnsupportedAttributeType {
                    attribute: name,
                    actual: format!("{actual:?}"),
                });
            }
        };
        Ok(WrfAttribute::new(name, value))
    }

    fn map_data_type(variable: &str, data_type: NcVariableType) -> WrfIoResult<WrfDataType> {
        match data_type {
            NcVariableType::Char => Ok(WrfDataType::Character),
            NcVariableType::Int(IntType::I32) => Ok(WrfDataType::Int32),
            NcVariableType::Float(FloatType::F32) => Ok(WrfDataType::Float32),
            NcVariableType::Float(FloatType::F64) => Ok(WrfDataType::Float64),
            actual => Err(WrfIoError::UnsupportedDataType {
                variable: variable.to_owned(),
                actual: format!("{actual:?}"),
            }),
        }
    }
}
