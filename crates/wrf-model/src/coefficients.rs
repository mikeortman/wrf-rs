use wrf_dynamics::{
    AcousticStepFinalizationCoefficients, AcousticTrajectoryCoefficients,
    DryTendencyAssemblyCoefficients, RungeKuttaPreparationCoefficients,
};

use crate::{ArwModelError, ArwModelResult};

/// Owned vertical coefficients shared across the accepted ARW stages.
pub struct ArwModelCoefficients {
    half_level_mass_multiplier: Vec<f32>,
    half_level_mass_offset: Vec<f32>,
    full_level_mass_multiplier: Vec<f32>,
    full_level_mass_offset: Vec<f32>,
    eta_level: Vec<f32>,
    half_level_eta_thickness: Vec<f32>,
    inverse_half_level_spacing: Vec<f32>,
    inverse_full_level_spacing: Vec<f32>,
    upper_full_level_weight: Vec<f32>,
    lower_full_level_weight: Vec<f32>,
}

impl ArwModelCoefficients {
    pub(crate) fn bottom_top_points(&self) -> usize {
        self.half_level_mass_multiplier.len()
    }

    /// Validates and owns the WRF hybrid-coordinate arrays used by this slice.
    ///
    /// # Errors
    ///
    /// Returns the first coefficient whose length differs from the padded
    /// bottom-top storage extent.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        bottom_top_points: usize,
        half_level_mass_multiplier: Vec<f32>,
        half_level_mass_offset: Vec<f32>,
        full_level_mass_multiplier: Vec<f32>,
        full_level_mass_offset: Vec<f32>,
        eta_level: Vec<f32>,
        half_level_eta_thickness: Vec<f32>,
        inverse_half_level_spacing: Vec<f32>,
        inverse_full_level_spacing: Vec<f32>,
        upper_full_level_weight: Vec<f32>,
        lower_full_level_weight: Vec<f32>,
    ) -> ArwModelResult<Self> {
        for (name, values) in [
            ("c1h", &half_level_mass_multiplier),
            ("c2h", &half_level_mass_offset),
            ("c1f", &full_level_mass_multiplier),
            ("c2f", &full_level_mass_offset),
            ("znu", &eta_level),
            ("dnw", &half_level_eta_thickness),
            ("rdnw", &inverse_half_level_spacing),
            ("rdn", &inverse_full_level_spacing),
            ("fnm", &upper_full_level_weight),
            ("fnp", &lower_full_level_weight),
        ] {
            if values.len() != bottom_top_points {
                return Err(ArwModelError::CoefficientLengthMismatch {
                    name,
                    expected: bottom_top_points,
                    actual: values.len(),
                });
            }
        }
        Ok(Self {
            half_level_mass_multiplier,
            half_level_mass_offset,
            full_level_mass_multiplier,
            full_level_mass_offset,
            eta_level,
            half_level_eta_thickness,
            inverse_half_level_spacing,
            inverse_full_level_spacing,
            upper_full_level_weight,
            lower_full_level_weight,
        })
    }

    pub(crate) fn runge_kutta(&self) -> RungeKuttaPreparationCoefficients<'_> {
        RungeKuttaPreparationCoefficients::new(
            &self.half_level_mass_multiplier,
            &self.half_level_mass_offset,
            &self.full_level_mass_multiplier,
            &self.full_level_mass_offset,
            &self.half_level_eta_thickness,
        )
    }

    pub(crate) fn dry_tendency(&self) -> DryTendencyAssemblyCoefficients<'_> {
        DryTendencyAssemblyCoefficients::new(
            &self.half_level_mass_multiplier,
            &self.half_level_mass_offset,
        )
    }

    pub(crate) fn acoustic(&self) -> AcousticTrajectoryCoefficients<'_> {
        AcousticTrajectoryCoefficients::new(
            &self.half_level_mass_multiplier,
            &self.half_level_mass_offset,
            &self.full_level_mass_multiplier,
            &self.full_level_mass_offset,
            &self.eta_level,
            &self.half_level_eta_thickness,
            &self.inverse_half_level_spacing,
            &self.inverse_full_level_spacing,
            &self.upper_full_level_weight,
            &self.lower_full_level_weight,
        )
    }

    pub(crate) fn finalization(&self) -> AcousticStepFinalizationCoefficients<'_> {
        AcousticStepFinalizationCoefficients::new(
            &self.half_level_mass_multiplier,
            &self.half_level_mass_offset,
            &self.full_level_mass_multiplier,
            &self.full_level_mass_offset,
        )
    }
}
