use std::ops::Range;

/// One disjoint half-open range assigned to a CPU worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearChunk {
    range: Range<usize>,
}

impl LinearChunk {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { range: start..end }
    }

    /// Returns the global half-open linear range represented by this chunk.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the number of scalar values in the chunk.
    pub fn value_count(&self) -> usize {
        self.range.len()
    }
}
