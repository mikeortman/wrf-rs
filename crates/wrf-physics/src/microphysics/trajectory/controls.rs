use crate::{ArwMicrophysicsError, ArwMicrophysicsResult};

/// Configurable scalar controlling the ARW microphysics trajectory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArwMicrophysicsControl {
    /// Physics timestep in seconds.
    TimeStep,
    /// Maximum magnitude of microphysics potential-temperature tendency in K/s.
    MaximumPotentialTemperatureTendency,
}

/// Validated controls for one ARW time-split microphysics trajectory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArwMicrophysicsControls {
    time_step: f32,
    uses_moist_potential_temperature: bool,
    is_heating_enabled: bool,
    maximum_potential_temperature_tendency: f32,
}

impl ArwMicrophysicsControls {
    /// Creates controls from WRF's `dt`, `use_theta_m`, `no_mp_heating`, and
    /// `mp_tend_lim` values.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the timestep is not finite and positive or the
    /// tendency limit is not finite and non-negative.
    pub fn try_new(
        time_step: f32,
        uses_moist_potential_temperature: bool,
        is_heating_enabled: bool,
        maximum_potential_temperature_tendency: f32,
    ) -> ArwMicrophysicsResult<Self> {
        validate_positive(ArwMicrophysicsControl::TimeStep, time_step)?;
        if !maximum_potential_temperature_tendency.is_finite()
            || maximum_potential_temperature_tendency < 0.0
        {
            return Err(ArwMicrophysicsError::InvalidControl {
                control: ArwMicrophysicsControl::MaximumPotentialTemperatureTendency,
                value: maximum_potential_temperature_tendency,
            });
        }
        Ok(Self {
            time_step,
            uses_moist_potential_temperature,
            is_heating_enabled,
            maximum_potential_temperature_tendency,
        })
    }

    /// Creates the pinned WRF v4.7.1 defaults for a supplied timestep.
    pub fn try_from_wrf_defaults(time_step: f32) -> ArwMicrophysicsResult<Self> {
        Self::try_new(time_step, true, true, 10.0)
    }

    pub(crate) const fn time_step(self) -> f32 {
        self.time_step
    }

    pub(crate) const fn uses_moist_potential_temperature(self) -> bool {
        self.uses_moist_potential_temperature
    }

    pub(crate) const fn is_heating_enabled(self) -> bool {
        self.is_heating_enabled
    }

    pub(crate) const fn maximum_potential_temperature_tendency(self) -> f32 {
        self.maximum_potential_temperature_tendency
    }
}

fn validate_positive(control: ArwMicrophysicsControl, value: f32) -> ArwMicrophysicsResult<()> {
    if value.is_finite() && value > 0.0 {
        return Ok(());
    }
    Err(ArwMicrophysicsError::InvalidControl { control, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_finite_non_positive_and_negative_controls() {
        for time_step in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(matches!(
                ArwMicrophysicsControls::try_from_wrf_defaults(time_step),
                Err(ArwMicrophysicsError::InvalidControl {
                    control: ArwMicrophysicsControl::TimeStep,
                    ..
                })
            ));
        }
        assert!(matches!(
            ArwMicrophysicsControls::try_new(60.0, true, true, -0.1),
            Err(ArwMicrophysicsError::InvalidControl {
                control: ArwMicrophysicsControl::MaximumPotentialTemperatureTendency,
                ..
            })
        ));
    }
}
