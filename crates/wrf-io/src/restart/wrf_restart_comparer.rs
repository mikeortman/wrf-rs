use std::path::Path;

use crate::restart::VariableChunkPlan;
use crate::{WrfFileKind, WrfIoError, WrfIoResult, WrfNetcdfReader, WrfVariableSchema};

const MAXIMUM_COMPARISON_BYTES: usize = 1024 * 1024;

/// Compares restart schema, metadata, and field bits with bounded scratch.
#[derive(Debug, Default)]
pub struct WrfRestartComparer;

impl WrfRestartComparer {
    /// Requires exact schema and primitive bits for trajectory-resumable state.
    ///
    /// Ordinary history output may use scientific tolerances. Restart files do
    /// not: a different stored state can start a different trajectory.
    pub fn compare_paths(
        left_path: impl AsRef<Path>,
        right_path: impl AsRef<Path>,
    ) -> WrfIoResult<()> {
        let left = WrfNetcdfReader::open(left_path)?;
        let right = WrfNetcdfReader::open(right_path)?;
        Self::require_restart(&left)?;
        Self::require_restart(&right)?;
        if left.schema() != right.schema() {
            return Err(WrfIoError::RestartSchemaMismatch);
        }

        for variable in left.schema().variables() {
            Self::compare_variable(&left, &right, variable)?;
        }
        Ok(())
    }

    fn require_restart(reader: &WrfNetcdfReader) -> WrfIoResult<()> {
        if reader.schema().file_kind() == WrfFileKind::Restart {
            return Ok(());
        }
        Err(WrfIoError::NotRestartFile {
            path: reader.path().to_path_buf(),
        })
    }

    fn compare_variable(
        left: &WrfNetcdfReader,
        right: &WrfNetcdfReader,
        variable: &WrfVariableSchema,
    ) -> WrfIoResult<()> {
        let lengths = variable
            .dimensions()
            .iter()
            .map(|name| {
                left.schema()
                    .dimensions()
                    .iter()
                    .find(|dimension| dimension.name() == *name)
                    .map(|dimension| dimension.length())
                    .ok_or_else(|| WrfIoError::UnsupportedDimension {
                        name: name.as_str().to_owned(),
                    })
            })
            .collect::<WrfIoResult<Vec<_>>>()?;
        let maximum_elements =
            (MAXIMUM_COMPARISON_BYTES / variable.data_type().byte_count()).max(1);
        let plan = VariableChunkPlan::try_new(variable.name(), &lengths, maximum_elements)?;
        let mut left_values = Vec::new();
        let mut right_values = Vec::new();

        for chunk in plan.chunks() {
            let byte_count = chunk.element_count * variable.data_type().byte_count();
            left_values.resize(byte_count, 0);
            right_values.resize(byte_count, 0);
            left.read_raw_chunk(
                variable.name(),
                &chunk.start,
                &chunk.count,
                &mut left_values,
            )?;
            right.read_raw_chunk(
                variable.name(),
                &chunk.start,
                &chunk.count,
                &mut right_values,
            )?;

            if let Some(byte_index) = left_values
                .iter()
                .zip(&right_values)
                .position(|(left, right)| left != right)
            {
                return Err(WrfIoError::RestartDataMismatch {
                    variable: variable.name().clone(),
                    element_index: chunk.element_offset
                        + byte_index / variable.data_type().byte_count(),
                });
            }
        }
        Ok(())
    }
}
