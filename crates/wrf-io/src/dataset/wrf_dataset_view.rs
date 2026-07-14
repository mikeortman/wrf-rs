use std::collections::HashSet;

use crate::{WrfFileSchema, WrfIoError, WrfIoResult, WrfVariableView};

/// Validated borrowed data matching one complete WRF file schema.
#[derive(Debug)]
pub struct WrfDatasetView<'a> {
    schema: &'a WrfFileSchema,
    variables: Vec<WrfVariableView<'a>>,
}

impl<'a> WrfDatasetView<'a> {
    /// Validates variable membership, type, and element counts before I/O.
    pub fn try_new(
        schema: &'a WrfFileSchema,
        variables: Vec<WrfVariableView<'a>>,
    ) -> WrfIoResult<Self> {
        let mut seen = HashSet::with_capacity(variables.len());
        for variable in &variables {
            let name = variable.name();
            if !seen.insert(name.clone()) {
                return Err(WrfIoError::DuplicateVariable {
                    variable: name.clone(),
                });
            }

            let variable_schema =
                schema
                    .variable(name)
                    .ok_or_else(|| WrfIoError::UnexpectedVariable {
                        variable: name.clone(),
                    })?;
            let actual_type = variable.values().data_type();
            if variable_schema.data_type() != actual_type {
                return Err(WrfIoError::VariableTypeMismatch {
                    variable: name.clone(),
                    expected: variable_schema.data_type(),
                    actual: actual_type,
                });
            }

            let expected_length = schema.variable_element_count(variable_schema)?;
            let actual_length = variable.values().len();
            if expected_length != actual_length {
                return Err(WrfIoError::VariableLengthMismatch {
                    variable: name.clone(),
                    expected: expected_length,
                    actual: actual_length,
                });
            }
        }

        for variable_schema in schema.variables() {
            if !seen.contains(variable_schema.name()) {
                return Err(WrfIoError::MissingVariable {
                    variable: variable_schema.name().clone(),
                });
            }
        }

        Ok(Self { schema, variables })
    }

    /// Returns the validated schema.
    pub const fn schema(&self) -> &WrfFileSchema {
        self.schema
    }

    /// Returns the complete validated variable views.
    pub fn variables(&self) -> &[WrfVariableView<'a>] {
        &self.variables
    }
}

#[cfg(test)]
mod tests {
    use crate::{WrfFileKind, WrfGridDimensions, WrfTimestamp, WrfVariableValues};

    use super::*;

    #[test]
    fn try_new_rejects_missing_variables_before_writer_creation() {
        let schema = WrfFileSchema::try_minimal_arw(
            WrfFileKind::Restart,
            WrfGridDimensions::try_new(2, 2, 2).unwrap(),
            WrfTimestamp::try_new("2000-01-01_00:00:00").unwrap(),
            WrfTimestamp::try_new("2000-01-01_00:00:00").unwrap(),
            1_000.0,
            1_000.0,
        )
        .unwrap();
        let only_times = WrfVariableView::try_new(
            "Times",
            WrfVariableValues::Character(b"2000-01-01_00:00:00"),
        )
        .unwrap();

        assert!(matches!(
            WrfDatasetView::try_new(&schema, vec![only_times]),
            Err(WrfIoError::MissingVariable { .. })
        ));
    }
}
