use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;

use netcdf3::{DataSet, FileWriter, Version};

use crate::{
    WrfAttribute, WrfAttributeValue, WrfDataType, WrfDatasetView, WrfIoError, WrfIoResult,
    WrfVariableValues, WrfVariableView,
};

const OUTPUT_BUFFER_BYTES: usize = 1024 * 1024;

/// Writes complete WRF datasets in NetCDF-3 64-bit-offset format.
///
/// WRF exposes this as `use_netcdf_classic`. The one-shot API validates all
/// fields before opening the path, borrows caller storage, and performs no
/// field-sized clone.
#[derive(Debug, Default)]
pub struct WrfNetcdfWriter;

impl WrfNetcdfWriter {
    /// Writes one complete validated initialization or restart dataset.
    pub fn write(path: impl AsRef<Path>, dataset: &WrfDatasetView<'_>) -> WrfIoResult<()> {
        let path = path.as_ref().to_path_buf();
        let definition = Self::build_definition(dataset)?;
        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(netcdf3::WriteError::from)
            .map_err(|source| WrfIoError::Netcdf3Write {
                path: path.clone(),
                source,
            })?;
        let buffered_output = BufWriter::with_capacity(OUTPUT_BUFFER_BYTES, output);
        let mut writer =
            FileWriter::open_seek_write(path.to_string_lossy().as_ref(), Box::new(buffered_output))
                .map_err(|source| WrfIoError::Netcdf3Write {
                    path: path.clone(),
                    source,
                })?;
        writer
            .set_def(&definition, Version::Offset64Bit, 0)
            .map_err(|source| WrfIoError::Netcdf3Write {
                path: path.clone(),
                source,
            })?;

        for variable in dataset.variables() {
            Self::write_variable(&mut writer, variable, &path)?;
        }

        writer
            .close()
            .map_err(|source| WrfIoError::Netcdf3Write { path, source })
    }

    fn build_definition(dataset: &WrfDatasetView<'_>) -> WrfIoResult<DataSet> {
        let mut definition = DataSet::new();
        for dimension in dataset.schema().dimensions() {
            let result = if dimension.is_unlimited() {
                definition.set_unlimited_dim(dimension.name().as_str(), dimension.length())
            } else {
                definition.add_fixed_dim(dimension.name().as_str(), dimension.length())
            };
            result.map_err(|source| WrfIoError::Netcdf3Schema { source })?;
        }

        for attribute in dataset.schema().attributes() {
            Self::add_global_attribute(&mut definition, attribute)?;
        }

        for variable in dataset.schema().variables() {
            let dimension_names: Vec<&str> = variable
                .dimensions()
                .iter()
                .map(|name| name.as_str())
                .collect();
            let result = match variable.data_type() {
                WrfDataType::Character => {
                    definition.add_var_u8(variable.name().as_str(), &dimension_names)
                }
                WrfDataType::Int32 => {
                    definition.add_var_i32(variable.name().as_str(), &dimension_names)
                }
                WrfDataType::Float32 => {
                    definition.add_var_f32(variable.name().as_str(), &dimension_names)
                }
                WrfDataType::Float64 => {
                    definition.add_var_f64(variable.name().as_str(), &dimension_names)
                }
            };
            result.map_err(|source| WrfIoError::Netcdf3Schema { source })?;

            for attribute in variable.attributes() {
                Self::add_variable_attribute(&mut definition, variable.name().as_str(), attribute)?;
            }
        }

        Ok(definition)
    }

    fn add_global_attribute(definition: &mut DataSet, attribute: &WrfAttribute) -> WrfIoResult<()> {
        let result = match attribute.value() {
            WrfAttributeValue::Text(value) => {
                definition.add_global_attr_string(attribute.name(), value)
            }
            WrfAttributeValue::Int32(values) => {
                definition.add_global_attr_i32(attribute.name(), values.clone())
            }
            WrfAttributeValue::Float32(values) => {
                definition.add_global_attr_f32(attribute.name(), values.clone())
            }
            WrfAttributeValue::Float64(values) => {
                definition.add_global_attr_f64(attribute.name(), values.clone())
            }
        };
        result
            .map(|_| ())
            .map_err(|source| WrfIoError::Netcdf3Schema { source })
    }

    fn add_variable_attribute(
        definition: &mut DataSet,
        variable_name: &str,
        attribute: &WrfAttribute,
    ) -> WrfIoResult<()> {
        let result = match attribute.value() {
            WrfAttributeValue::Text(value) => {
                definition.add_var_attr_string(variable_name, attribute.name(), value)
            }
            WrfAttributeValue::Int32(values) => {
                definition.add_var_attr_i32(variable_name, attribute.name(), values.clone())
            }
            WrfAttributeValue::Float32(values) => {
                definition.add_var_attr_f32(variable_name, attribute.name(), values.clone())
            }
            WrfAttributeValue::Float64(values) => {
                definition.add_var_attr_f64(variable_name, attribute.name(), values.clone())
            }
        };
        result
            .map(|_| ())
            .map_err(|source| WrfIoError::Netcdf3Schema { source })
    }

    fn write_variable(
        writer: &mut FileWriter<'_>,
        variable: &WrfVariableView<'_>,
        path: &Path,
    ) -> WrfIoResult<()> {
        let result = match variable.values() {
            WrfVariableValues::Character(values) => {
                writer.write_var_u8(variable.name().as_str(), values)
            }
            WrfVariableValues::Int32(values) => {
                writer.write_var_i32(variable.name().as_str(), values)
            }
            WrfVariableValues::Float32(values) => {
                writer.write_var_f32(variable.name().as_str(), values)
            }
            WrfVariableValues::Float64(values) => {
                writer.write_var_f64(variable.name().as_str(), values)
            }
        };
        result.map_err(|source| WrfIoError::Netcdf3Write {
            path: path.to_path_buf(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        WrfFileKind, WrfFileSchema, WrfGridDimensions, WrfNetcdfReader, WrfRestartComparer,
        WrfTimestamp, WrfVariableName, WrfVariableValues, WrfVariableView,
    };

    use super::*;

    #[test]
    fn writer_and_reader_preserve_exact_schema_and_field_bits() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("minimal_restart.nc");
        let schema = fixture_schema(WrfFileKind::Restart);
        let times = *b"2000-09-18_16:42:01";
        let u = values(30, 100);
        let v = values(32, 200);
        let w = values(36, 300);
        let ph = values(36, 400);
        let phb = values(36, 500);
        let mut temperature = values(24, 600);
        let mu = values(12, 700);
        let mub = values(12, 800);
        let pressure = values(24, 900);
        let base_pressure = values(24, 1_000);
        let water_vapor = values(24, 1_100);
        let model_minutes = [60.0_f32];
        let dataset = WrfDatasetView::try_new(
            &schema,
            vec![
                view("Times", WrfVariableValues::Character(&times)),
                view("U", WrfVariableValues::Float32(&u)),
                view("V", WrfVariableValues::Float32(&v)),
                view("W", WrfVariableValues::Float32(&w)),
                view("PH", WrfVariableValues::Float32(&ph)),
                view("PHB", WrfVariableValues::Float32(&phb)),
                view("THM", WrfVariableValues::Float32(&temperature)),
                view("MU", WrfVariableValues::Float32(&mu)),
                view("MUB", WrfVariableValues::Float32(&mub)),
                view("P", WrfVariableValues::Float32(&pressure)),
                view("PB", WrfVariableValues::Float32(&base_pressure)),
                view("QVAPOR", WrfVariableValues::Float32(&water_vapor)),
                view("XTIME", WrfVariableValues::Float32(&model_minutes)),
            ],
        )
        .unwrap();

        WrfNetcdfWriter::write(&path, &dataset).unwrap();

        let reader = WrfNetcdfReader::open(&path).unwrap();
        assert_eq!(reader.schema(), &schema);
        let mut actual_temperature = vec![0.0_f32; temperature.len()];
        reader
            .read_f32_into(
                &WrfVariableName::try_new("THM").unwrap(),
                &mut actual_temperature,
            )
            .unwrap();
        assert!(
            actual_temperature
                .iter()
                .zip(&temperature)
                .all(|(actual, expected)| actual.to_bits() == expected.to_bits())
        );
        assert!(matches!(
            reader.read_f32_into(
                &WrfVariableName::try_new("THM").unwrap(),
                &mut actual_temperature[..23],
            ),
            Err(WrfIoError::VariableLengthMismatch {
                expected: 24,
                actual: 23,
                ..
            })
        ));
        let mut wrong_primitive = vec![0_i32; 24];
        assert!(matches!(
            reader.read_i32_into(
                &WrfVariableName::try_new("THM").unwrap(),
                &mut wrong_primitive,
            ),
            Err(WrfIoError::VariableTypeMismatch { .. })
        ));
        WrfRestartComparer::compare_paths(&path, &path).unwrap();

        drop(dataset);
        temperature[3] = f32::from_bits(temperature[3].to_bits() + 1);
        let changed_path = directory.path().join("changed_restart.nc");
        let changed_dataset = WrfDatasetView::try_new(
            &schema,
            vec![
                view("Times", WrfVariableValues::Character(&times)),
                view("U", WrfVariableValues::Float32(&u)),
                view("V", WrfVariableValues::Float32(&v)),
                view("W", WrfVariableValues::Float32(&w)),
                view("PH", WrfVariableValues::Float32(&ph)),
                view("PHB", WrfVariableValues::Float32(&phb)),
                view("THM", WrfVariableValues::Float32(&temperature)),
                view("MU", WrfVariableValues::Float32(&mu)),
                view("MUB", WrfVariableValues::Float32(&mub)),
                view("P", WrfVariableValues::Float32(&pressure)),
                view("PB", WrfVariableValues::Float32(&base_pressure)),
                view("QVAPOR", WrfVariableValues::Float32(&water_vapor)),
                view("XTIME", WrfVariableValues::Float32(&model_minutes)),
            ],
        )
        .unwrap();
        WrfNetcdfWriter::write(&changed_path, &changed_dataset).unwrap();
        assert!(matches!(
            WrfRestartComparer::compare_paths(&path, &changed_path),
            Err(WrfIoError::RestartDataMismatch {
                element_index: 3,
                ..
            })
        ));
    }

    fn fixture_schema(file_kind: WrfFileKind) -> WrfFileSchema {
        WrfFileSchema::try_minimal_arw(
            file_kind,
            WrfGridDimensions::try_new(4, 3, 2).unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:42:01").unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:00:00").unwrap(),
            12_000.0,
            12_000.0,
        )
        .unwrap()
    }

    fn values(length: usize, offset: usize) -> Vec<f32> {
        (0..length).map(|index| (offset + index) as f32).collect()
    }

    fn view<'a>(name: &str, values: WrfVariableValues<'a>) -> WrfVariableView<'a> {
        WrfVariableView::try_new(name, values).unwrap()
    }
}
