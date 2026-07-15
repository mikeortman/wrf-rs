use crate::{
    AcousticStepFinalizationError, AcousticStepFinalizationPhase, AcousticStepFinalizationResult,
};

/// Scalar and phase controls for WRF `small_step_finish`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AcousticStepFinalizationControls {
    pub(crate) acoustic_substep_count: usize,
    pub(crate) acoustic_time_step: f32,
    pub(crate) phase: AcousticStepFinalizationPhase,
}

impl AcousticStepFinalizationControls {
    /// Returns the number of completed acoustic substeps.
    pub const fn acoustic_substep_count(self) -> usize {
        self.acoustic_substep_count
    }

    /// Returns the acoustic timestep in seconds.
    pub const fn acoustic_time_step(self) -> f32 {
        self.acoustic_time_step
    }

    /// Returns the Runge-Kutta finalization phase.
    pub const fn phase(self) -> AcousticStepFinalizationPhase {
        self.phase
    }

    /// Validates a nonempty acoustic sequence and preserves IEEE timestep bits.
    ///
    /// # Errors
    ///
    /// Returns [`AcousticStepFinalizationError::ZeroSubstepCount`] for an empty
    /// sequence, which cannot arise in the accepted acoustic trajectory.
    pub fn try_new(
        acoustic_substep_count: usize,
        acoustic_time_step: f32,
        phase: AcousticStepFinalizationPhase,
    ) -> AcousticStepFinalizationResult<Self> {
        if acoustic_substep_count == 0 {
            return Err(AcousticStepFinalizationError::ZeroSubstepCount);
        }
        Ok(Self {
            acoustic_substep_count,
            acoustic_time_step,
            phase,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_empty_acoustic_sequence() {
        assert_eq!(
            AcousticStepFinalizationControls::try_new(
                0,
                0.25,
                AcousticStepFinalizationPhase::Intermediate,
            ),
            Err(AcousticStepFinalizationError::ZeroSubstepCount)
        );
    }
}
