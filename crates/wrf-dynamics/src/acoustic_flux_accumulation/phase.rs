use crate::{AcousticFluxAccumulationError, AcousticFluxAccumulationResult};

/// One-based position within an acoustic small-step sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcousticSubstepPhase {
    iteration: usize,
    count: usize,
}

impl AcousticSubstepPhase {
    /// Validates a one-based iteration and the total number of substeps.
    pub fn try_new(iteration: usize, count: usize) -> AcousticFluxAccumulationResult<Self> {
        if count == 0 {
            return Err(AcousticFluxAccumulationError::ZeroSubstepCount);
        }
        if iteration == 0 || iteration > count {
            return Err(AcousticFluxAccumulationError::SubstepOutOfRange { iteration, count });
        }
        Ok(Self { iteration, count })
    }

    /// Returns the one-based current iteration.
    pub const fn iteration(self) -> usize {
        self.iteration
    }

    /// Returns the total acoustic-substep count.
    pub const fn count(self) -> usize {
        self.count
    }

    pub(crate) const fn is_first(self) -> bool {
        self.iteration == 1
    }

    pub(crate) const fn is_last(self) -> bool {
        self.iteration == self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_count_and_out_of_range_iterations() {
        assert_eq!(
            AcousticSubstepPhase::try_new(1, 0),
            Err(AcousticFluxAccumulationError::ZeroSubstepCount)
        );
        assert_eq!(
            AcousticSubstepPhase::try_new(0, 3),
            Err(AcousticFluxAccumulationError::SubstepOutOfRange {
                iteration: 0,
                count: 3,
            })
        );
        assert_eq!(
            AcousticSubstepPhase::try_new(4, 3),
            Err(AcousticFluxAccumulationError::SubstepOutOfRange {
                iteration: 4,
                count: 3,
            })
        );
    }
}
