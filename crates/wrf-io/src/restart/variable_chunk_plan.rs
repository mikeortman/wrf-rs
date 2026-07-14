use crate::{WrfIoError, WrfIoResult, WrfVariableName};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VariableChunk {
    pub(crate) start: Vec<usize>,
    pub(crate) count: Vec<usize>,
    pub(crate) element_offset: usize,
    pub(crate) element_count: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct VariableChunkPlan {
    chunks: Vec<VariableChunk>,
}

impl VariableChunkPlan {
    pub(crate) fn try_new(
        variable: &WrfVariableName,
        lengths: &[usize],
        maximum_elements: usize,
    ) -> WrfIoResult<Self> {
        let total_elements = lengths.iter().try_fold(1_usize, |count, &length| {
            count
                .checked_mul(length)
                .ok_or_else(|| WrfIoError::ElementCountOverflow {
                    variable: variable.clone(),
                })
        })?;
        if total_elements <= maximum_elements {
            return Ok(Self {
                chunks: vec![VariableChunk {
                    start: vec![0; lengths.len()],
                    count: lengths.to_vec(),
                    element_offset: 0,
                    element_count: total_elements,
                }],
            });
        }

        let mut trailing_elements = 1_usize;
        let mut split_axis = lengths.len().saturating_sub(1);
        for axis in (0..lengths.len()).rev() {
            let with_axis = lengths[axis]
                .checked_mul(trailing_elements)
                .ok_or_else(|| WrfIoError::ElementCountOverflow {
                    variable: variable.clone(),
                })?;
            if with_axis > maximum_elements {
                split_axis = axis;
                break;
            }
            trailing_elements = with_axis;
        }

        let split_length = lengths[split_axis];
        let split_chunk_length = (maximum_elements / trailing_elements).max(1);
        let prefix_count = lengths[..split_axis]
            .iter()
            .try_fold(1_usize, |count, &length| {
                count
                    .checked_mul(length)
                    .ok_or_else(|| WrfIoError::ElementCountOverflow {
                        variable: variable.clone(),
                    })
            })?;
        let chunks_per_prefix = split_length.div_ceil(split_chunk_length);
        let capacity = prefix_count.checked_mul(chunks_per_prefix).ok_or_else(|| {
            WrfIoError::ElementCountOverflow {
                variable: variable.clone(),
            }
        })?;
        let mut chunks = Vec::with_capacity(capacity);

        for prefix_linear_index in 0..prefix_count {
            let mut prefix_remainder = prefix_linear_index;
            let mut start = vec![0; lengths.len()];
            for axis in (0..split_axis).rev() {
                start[axis] = prefix_remainder % lengths[axis];
                prefix_remainder /= lengths[axis];
            }

            for split_start in (0..split_length).step_by(split_chunk_length) {
                let count_on_split_axis = split_chunk_length.min(split_length - split_start);
                start[split_axis] = split_start;
                let mut count = lengths.to_vec();
                count[..split_axis].fill(1);
                count[split_axis] = count_on_split_axis;
                let element_count = count_on_split_axis * trailing_elements;
                let element_offset = prefix_linear_index * split_length * trailing_elements
                    + split_start * trailing_elements;
                chunks.push(VariableChunk {
                    start: start.clone(),
                    count,
                    element_offset,
                    element_count,
                });
            }
        }

        Ok(Self { chunks })
    }

    pub(crate) fn chunks(&self) -> &[VariableChunk] {
        &self.chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_bounds_buffers_and_covers_row_major_elements_once() {
        let variable = WrfVariableName::try_new("T").unwrap();
        let plan = VariableChunkPlan::try_new(&variable, &[2, 5, 3, 4], 48).unwrap();

        assert_eq!(plan.chunks().len(), 4);
        assert!(plan.chunks().iter().all(|chunk| chunk.element_count <= 48));
        assert_eq!(plan.chunks().first().unwrap().element_offset, 0);
        assert_eq!(
            plan.chunks().last().unwrap().element_offset
                + plan.chunks().last().unwrap().element_count,
            120
        );
    }
}
