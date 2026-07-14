use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::{WrfDataType, WrfVariableName};

/// Failures produced by typed WRF schema and NetCDF operations.
#[derive(Debug)]
pub enum WrfIoError {
    /// A timestamp does not use WRF's fixed 19-byte representation.
    InvalidTimestamp {
        /// The rejected timestamp.
        value: String,
    },
    /// A required dimension is empty.
    EmptyDimension {
        /// The empty dimension's WRF name.
        name: &'static str,
    },
    /// A staggered dimension cannot be represented.
    DimensionLengthOverflow {
        /// The dimension's WRF name.
        name: &'static str,
        /// The unrepresentable dimension length.
        length: usize,
    },
    /// Grid spacing is non-finite or non-positive.
    InvalidGridSpacing {
        /// The grid axis whose spacing is invalid.
        axis: &'static str,
        /// The rejected spacing in meters.
        value: f32,
    },
    /// A variable name is not valid NetCDF syntax.
    InvalidVariableName {
        /// The rejected variable name.
        value: String,
    },
    /// A dimension name is not valid NetCDF syntax.
    InvalidDimensionName {
        /// The rejected dimension name.
        value: String,
    },
    /// A variable references a dimension the schema does not define.
    UnsupportedDimension {
        /// The undefined dimension name.
        name: String,
    },
    /// A state references a dimension symbol with no `dimspec` entry.
    UnknownRegistryDimension {
        /// The state whose dimensions could not be resolved.
        state: String,
        /// The unresolved Registry dimension symbol.
        dimension: String,
    },
    /// A namelist-bounded dimension has no caller-supplied value.
    MissingNamelistDimensionLength {
        /// The Registry dimension symbol.
        dimension: String,
        /// The namelist entry that must be supplied to the builder.
        namelist: String,
    },
    /// A Registry dimension resolves to a non-positive length.
    EmptyRegistryDimension {
        /// The Registry dimension symbol.
        dimension: String,
        /// The resolved inclusive start.
        start: i64,
        /// The resolved inclusive end.
        end: i64,
    },
    /// A Registry dimension's inclusive bounds cannot fit in a file length.
    RegistryDimensionLengthOverflow {
        /// The Registry dimension symbol.
        dimension: String,
        /// The resolved inclusive start.
        start: i64,
        /// The resolved inclusive end.
        end: i64,
    },
    /// A `standard_domain` dimension uses the constant coordinate axis.
    UnsupportedStandardDomainAxis {
        /// The Registry dimension symbol.
        dimension: String,
    },
    /// A state's dimension axes have no supported WRF external order.
    UnsupportedMemoryOrder {
        /// The state whose axes cannot be reordered.
        state: String,
        /// The Registry memory order that has no external reordering.
        memory_order: String,
    },
    /// A state's Registry value type has no restart NetCDF mapping.
    UnsupportedRegistryValueType {
        /// The state with the unsupported value type.
        state: String,
        /// The Registry value type spelling.
        value_type: String,
    },
    /// A state uses subgrid dimensions, which this slice does not port.
    UnsupportedSubgridDimensions {
        /// The state with subgrid-marked dimensions.
        state: String,
    },
    /// A selected state is a boundary array whose companions are not ported.
    UnsupportedBoundaryArray {
        /// The unsupported boundary-array state name.
        state: String,
    },
    /// WRF does not register logical arrays with three or more dimensions.
    UnsupportedLogicalFieldDimensions {
        /// The unsupported logical state name.
        state: String,
        /// The number of Registry dimensions.
        dimensions: usize,
    },
    /// WRF excludes processor-transposed state from external I/O.
    UnsupportedProcessorOrientation {
        /// The unsupported state name.
        state: String,
        /// The selected processor orientation.
        orientation: &'static str,
    },
    /// A selected state belongs to an out-of-scope four-dimensional bundle.
    UnsupportedScalarArrayMember {
        /// The unsupported state name.
        state: String,
    },
    /// A Registry dimension collides with WRF's record dimension.
    ReservedRegistryDimensionName {
        /// The reserved file dimension name.
        dimension: String,
    },
    /// A state's Registry I/O specification cannot be parsed.
    InvalidIoSpecification {
        /// The state with the malformed I/O specification.
        state: String,
        /// The rejected I/O specification.
        value: String,
    },
    /// Two states require the same named dimension with different lengths.
    DimensionLengthConflict {
        /// The conflicting file dimension name.
        dimension: String,
        /// The previously registered length.
        existing: usize,
        /// The newly requested length.
        requested: usize,
    },
    /// WRF's fixed NetCDF dimension table has no unused slot.
    RegistryDimensionTableFull {
        /// The pinned maximum number of fixed-dimension slots.
        maximum: usize,
    },
    /// The file uses a primitive outside this minimum WRF schema slice.
    UnsupportedDataType {
        /// The variable using the unsupported type.
        variable: String,
        /// The NetCDF type reported by the reader.
        actual: String,
    },
    /// The file uses an attribute representation outside this schema slice.
    UnsupportedAttributeType {
        /// The attribute using the unsupported type.
        attribute: String,
        /// The NetCDF type reported by the reader.
        actual: String,
    },
    /// The schema contains a duplicate variable.
    DuplicateVariable {
        /// The repeated variable name.
        variable: WrfVariableName,
    },
    /// Dataset data omitted a schema variable.
    MissingVariable {
        /// The missing variable name.
        variable: WrfVariableName,
    },
    /// Dataset data contains a variable absent from the schema.
    UnexpectedVariable {
        /// The unexpected variable name.
        variable: WrfVariableName,
    },
    /// A variable's declared and provided primitive types differ.
    VariableTypeMismatch {
        /// The variable with mismatched data.
        variable: WrfVariableName,
        /// The primitive required by the schema.
        expected: WrfDataType,
        /// The primitive supplied by the dataset.
        actual: WrfDataType,
    },
    /// A variable buffer does not match its schema element count.
    VariableLengthMismatch {
        /// The variable with the wrong element count.
        variable: WrfVariableName,
        /// The element count required by the schema.
        expected: usize,
        /// The supplied element count.
        actual: usize,
    },
    /// A variable could not be found in an opened file.
    VariableNotFound {
        /// The missing variable name.
        variable: WrfVariableName,
    },
    /// NetCDF-3 rejected a typed schema definition.
    Netcdf3Schema {
        /// The schema validation failure from the writer dependency.
        source: netcdf3::InvalidDataSet,
    },
    /// NetCDF-3 could not write a file.
    Netcdf3Write {
        /// The output path that could not be written.
        path: PathBuf,
        /// The writer dependency's failure.
        source: netcdf3::WriteError,
    },
    /// NetCDF-C could not open or inspect a file.
    NetcdfRead {
        /// The input path that could not be read.
        path: PathBuf,
        /// The variable being read, or `None` for file-level operations.
        variable: Option<WrfVariableName>,
        /// The NetCDF-C-backed reader failure.
        source: netcdf::Error,
    },
    /// Restart comparison requires files marked as restart datasets.
    NotRestartFile {
        /// The file lacking WRF's restart marker.
        path: PathBuf,
    },
    /// Restart schemas or metadata differ.
    RestartSchemaMismatch,
    /// Restart variable bits differ at one element.
    RestartDataMismatch {
        /// The variable containing the first mismatch.
        variable: WrfVariableName,
        /// The zero-based row-major element index of the first mismatch.
        element_index: usize,
    },
    /// An element-count computation overflowed.
    ElementCountOverflow {
        /// The variable whose dimensions overflowed `usize`.
        variable: WrfVariableName,
    },
}

/// Result alias for the WRF I/O boundary.
pub type WrfIoResult<T> = Result<T, WrfIoError>;

impl fmt::Display for WrfIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimestamp { value } => write!(
                formatter,
                "WRF timestamp {value:?} must use YYYY-MM-DD_HH:MM:SS"
            ),
            Self::EmptyDimension { name } => write!(formatter, "WRF dimension {name} is empty"),
            Self::DimensionLengthOverflow { name, length } => write!(
                formatter,
                "WRF dimension {name} length {length} cannot be staggered"
            ),
            Self::InvalidGridSpacing { axis, value } => {
                write!(
                    formatter,
                    "WRF {axis} grid spacing must be positive and finite, got {value}"
                )
            }
            Self::InvalidVariableName { value } => {
                write!(formatter, "{value:?} is not a valid NetCDF variable name")
            }
            Self::InvalidDimensionName { value } => {
                write!(formatter, "{value:?} is not a valid NetCDF dimension name")
            }
            Self::UnsupportedDimension { name } => {
                write!(
                    formatter,
                    "WRF dimension {name:?} is not defined by the file schema"
                )
            }
            Self::UnknownRegistryDimension { state, dimension } => write!(
                formatter,
                "Registry state {state} references dimension {dimension:?} with no dimspec entry"
            ),
            Self::MissingNamelistDimensionLength {
                dimension,
                namelist,
            } => write!(
                formatter,
                "Registry dimension {dimension:?} needs a length for namelist entry {namelist}"
            ),
            Self::EmptyRegistryDimension {
                dimension,
                start,
                end,
            } => write!(
                formatter,
                "Registry dimension {dimension:?} bounds {start}:{end} are not a positive length"
            ),
            Self::RegistryDimensionLengthOverflow {
                dimension,
                start,
                end,
            } => write!(
                formatter,
                "Registry dimension {dimension:?} bounds {start}:{end} cannot fit in a file dimension"
            ),
            Self::UnsupportedStandardDomainAxis { dimension } => write!(
                formatter,
                "Registry dimension {dimension:?} is standard_domain on the constant axis"
            ),
            Self::UnsupportedMemoryOrder {
                state,
                memory_order,
            } => write!(
                formatter,
                "Registry state {state} memory order {memory_order:?} has no WRF external order"
            ),
            Self::UnsupportedRegistryValueType { state, value_type } => write!(
                formatter,
                "Registry state {state} value type {value_type} has no restart NetCDF mapping"
            ),
            Self::UnsupportedSubgridDimensions { state } => write!(
                formatter,
                "Registry state {state} uses subgrid dimensions outside this slice"
            ),
            Self::UnsupportedBoundaryArray { state } => write!(
                formatter,
                "Registry state {state} is a boundary array outside this slice"
            ),
            Self::UnsupportedLogicalFieldDimensions { state, dimensions } => write!(
                formatter,
                "Registry logical state {state} has {dimensions} dimensions and is not registered for WRF I/O"
            ),
            Self::UnsupportedProcessorOrientation { state, orientation } => write!(
                formatter,
                "Registry state {state} has processor orientation {orientation}, which WRF excludes from I/O"
            ),
            Self::UnsupportedScalarArrayMember { state } => write!(
                formatter,
                "Registry state {state} belongs to a four-dimensional scalar bundle outside this slice"
            ),
            Self::ReservedRegistryDimensionName { dimension } => write!(
                formatter,
                "Registry dimension name {dimension:?} is reserved by WRF's NetCDF writer"
            ),
            Self::InvalidIoSpecification { state, value } => write!(
                formatter,
                "Registry state {state} has malformed I/O specification {value:?}"
            ),
            Self::DimensionLengthConflict {
                dimension,
                existing,
                requested,
            } => write!(
                formatter,
                "WRF dimension {dimension:?} is defined with length {existing} but requested with {requested}"
            ),
            Self::RegistryDimensionTableFull { maximum } => write!(
                formatter,
                "WRF NetCDF dimension table is limited to {maximum} fixed-dimension slots"
            ),
            Self::UnsupportedDataType { variable, actual } => write!(
                formatter,
                "WRF variable {variable} uses unsupported NetCDF type {actual}"
            ),
            Self::UnsupportedAttributeType { attribute, actual } => write!(
                formatter,
                "WRF attribute {attribute} uses unsupported NetCDF type {actual}"
            ),
            Self::DuplicateVariable { variable } => {
                write!(
                    formatter,
                    "WRF dataset contains duplicate variable {variable}"
                )
            }
            Self::MissingVariable { variable } => {
                write!(formatter, "WRF dataset is missing variable {variable}")
            }
            Self::UnexpectedVariable { variable } => {
                write!(
                    formatter,
                    "WRF dataset contains unexpected variable {variable}"
                )
            }
            Self::VariableTypeMismatch {
                variable,
                expected,
                actual,
            } => write!(
                formatter,
                "WRF variable {variable} expected {expected:?} data but received {actual:?}"
            ),
            Self::VariableLengthMismatch {
                variable,
                expected,
                actual,
            } => write!(
                formatter,
                "WRF variable {variable} expected {expected} values but received {actual}"
            ),
            Self::VariableNotFound { variable } => {
                write!(formatter, "WRF variable {variable} was not found")
            }
            Self::Netcdf3Schema { source } => {
                write!(formatter, "NetCDF-3 rejected the WRF schema: {source}")
            }
            Self::Netcdf3Write { path, source } => {
                write!(
                    formatter,
                    "failed to write WRF file {}: {source:?}",
                    path.display()
                )
            }
            Self::NetcdfRead {
                path,
                variable,
                source,
            } => {
                if let Some(variable) = variable {
                    write!(
                        formatter,
                        "failed to read WRF variable {variable} from {}: {source}",
                        path.display()
                    )
                } else {
                    write!(
                        formatter,
                        "failed to read WRF file {}: {source}",
                        path.display()
                    )
                }
            }
            Self::NotRestartFile { path } => {
                write!(
                    formatter,
                    "WRF file {} is not marked as a restart",
                    path.display()
                )
            }
            Self::RestartSchemaMismatch => formatter.write_str("WRF restart schemas differ"),
            Self::RestartDataMismatch {
                variable,
                element_index,
            } => write!(
                formatter,
                "WRF restart variable {variable} differs at element {element_index}"
            ),
            Self::ElementCountOverflow { variable } => {
                write!(
                    formatter,
                    "WRF variable {variable} element count overflowed"
                )
            }
        }
    }
}

impl Error for WrfIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Netcdf3Schema { source } => Some(source),
            Self::NetcdfRead { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_retains_variable_and_dimension_context() {
        let variable = WrfVariableName::try_new("QVAPOR").unwrap();
        let error = WrfIoError::VariableLengthMismatch {
            variable,
            expected: 24,
            actual: 23,
        };

        assert_eq!(
            error.to_string(),
            "WRF variable QVAPOR expected 24 values but received 23"
        );
    }
}
