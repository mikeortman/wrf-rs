use wrf_registry::{
    RegistryDocument, RegistryValueType, ResolvedScalarArrayLayout, RuntimeConfigurationChoice,
    StateStaggering, StateVariable,
};

use crate::{ArwModelError, ArwModelResult};

/// Validated Registry selection for the accepted ARW/Kessler trajectory.
///
/// The binding proves that every restart-owned ordinary field has the expected
/// WRF rank, time-level count, and staggering, and that `mp_physics=1` resolves
/// one ordered `moist` scalar layout. It owns only metadata; numerical storage
/// remains in [`crate::ArwModelState`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArwRegistryBinding {
    moisture_layout: ResolvedScalarArrayLayout,
}

impl ArwRegistryBinding {
    /// Binds the required ARW state and selected Kessler moisture package.
    ///
    /// # Errors
    ///
    /// Returns a typed error for missing, duplicate, wrongly typed, wrongly
    /// ranked, wrongly staggered, or insufficiently time-leveled state, and
    /// for malformed Registry package resolution.
    pub fn try_new(
        document: &RegistryDocument,
        configuration_choices: &[RuntimeConfigurationChoice],
    ) -> ArwModelResult<Self> {
        for specification in REQUIRED_STATE {
            validate_state(document, specification)?;
        }

        let mut moisture_layouts = document
            .resolve_scalar_array_layouts(configuration_choices)?
            .into_iter()
            .filter(|layout| layout.scalar_array_name() == "moist")
            .collect::<Vec<_>>();
        if moisture_layouts.len() != 1 {
            return Err(ArwModelError::MoistureLayoutCount {
                actual: moisture_layouts.len(),
            });
        }
        let moisture_layout = moisture_layouts
            .pop()
            .ok_or(ArwModelError::MoistureLayoutCount { actual: 0 })?;
        Ok(Self { moisture_layout })
    }

    /// Returns the Registry-ordered dense moisture layout used by physics.
    #[must_use]
    pub const fn moisture_layout(&self) -> &ResolvedScalarArrayLayout {
        &self.moisture_layout
    }
}

#[derive(Clone, Copy)]
struct RequiredState {
    name: &'static str,
    dimensions: &'static [&'static str],
    time_levels: u8,
    staggering: ExpectedStaggering,
}

#[derive(Clone, Copy)]
enum ExpectedStaggering {
    None,
    WestEast,
    SouthNorth,
    Vertical,
}

impl ExpectedStaggering {
    const fn description(self) -> &'static str {
        match self {
            Self::None => "unstaggered",
            Self::WestEast => "X staggered",
            Self::SouthNorth => "Y staggered",
            Self::Vertical => "Z staggered",
        }
    }

    const fn matches(self, actual: StateStaggering) -> bool {
        match self {
            Self::None => {
                !actual.is_x_staggered() && !actual.is_y_staggered() && !actual.is_z_staggered()
            }
            Self::WestEast => {
                actual.is_x_staggered() && !actual.is_y_staggered() && !actual.is_z_staggered()
            }
            Self::SouthNorth => {
                !actual.is_x_staggered() && actual.is_y_staggered() && !actual.is_z_staggered()
            }
            Self::Vertical => {
                !actual.is_x_staggered() && !actual.is_y_staggered() && actual.is_z_staggered()
            }
        }
    }
}

const REQUIRED_STATE: &[RequiredState] = &[
    RequiredState {
        name: "u",
        dimensions: &["i", "k", "j"],
        time_levels: 2,
        staggering: ExpectedStaggering::WestEast,
    },
    RequiredState {
        name: "v",
        dimensions: &["i", "k", "j"],
        time_levels: 2,
        staggering: ExpectedStaggering::SouthNorth,
    },
    RequiredState {
        name: "w",
        dimensions: &["i", "k", "j"],
        time_levels: 2,
        staggering: ExpectedStaggering::Vertical,
    },
    RequiredState {
        name: "t",
        dimensions: &["i", "k", "j"],
        time_levels: 2,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "ph",
        dimensions: &["i", "k", "j"],
        time_levels: 2,
        staggering: ExpectedStaggering::Vertical,
    },
    RequiredState {
        name: "mu",
        dimensions: &["i", "j"],
        time_levels: 2,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "mub",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "p",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "al",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "alb",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "pb",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "phb",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::Vertical,
    },
    RequiredState {
        name: "rainnc",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "rainncv",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "ww",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::Vertical,
    },
    RequiredState {
        name: "ww_m",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::Vertical,
    },
    RequiredState {
        name: "php",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "h_diabatic",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "qv_diabatic",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "qc_diabatic",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "rho",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "th_phy_m_t0",
        dimensions: &["i", "k", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "msfux",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::WestEast,
    },
    RequiredState {
        name: "msfuy",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::WestEast,
    },
    RequiredState {
        name: "msfvx",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::SouthNorth,
    },
    RequiredState {
        name: "msfvx_inv",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::SouthNorth,
    },
    RequiredState {
        name: "msfvy",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::SouthNorth,
    },
    RequiredState {
        name: "msftx",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "msfty",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
    RequiredState {
        name: "ht",
        dimensions: &["i", "j"],
        time_levels: 1,
        staggering: ExpectedStaggering::None,
    },
];

fn validate_state(
    document: &RegistryDocument,
    specification: &RequiredState,
) -> ArwModelResult<()> {
    let matching = document
        .state_variables()
        .filter(|state| state.name() == specification.name)
        .collect::<Vec<_>>();
    let state = match matching.as_slice() {
        [] => {
            return Err(ArwModelError::MissingRegistryState {
                name: specification.name,
            });
        }
        [state] => *state,
        _ => {
            return Err(ArwModelError::DuplicateRegistryState {
                name: specification.name,
            });
        }
    };
    validate_state_contract(state, specification)
}

fn validate_state_contract(
    state: &StateVariable,
    specification: &RequiredState,
) -> ArwModelResult<()> {
    if state.value_type() != &RegistryValueType::Real {
        return Err(ArwModelError::RegistryValueTypeMismatch {
            name: specification.name,
            actual: state.value_type().clone(),
        });
    }
    let actual_dimensions = state.dimensions().names();
    if actual_dimensions.len() != specification.dimensions.len() {
        return Err(ArwModelError::RegistryDimensionCountMismatch {
            name: specification.name,
            expected: specification.dimensions.len(),
            actual: actual_dimensions.len(),
        });
    }
    if !actual_dimensions
        .iter()
        .map(String::as_str)
        .eq(specification.dimensions.iter().copied())
    {
        return Err(ArwModelError::RegistryDimensionsMismatch {
            name: specification.name,
            expected: specification.dimensions,
            actual: actual_dimensions.to_vec(),
        });
    }
    let actual_time_levels = state.time_levels().get();
    if actual_time_levels != specification.time_levels {
        return Err(ArwModelError::RegistryTimeLevelMismatch {
            field: registry_field_for(specification),
            expected: specification.time_levels,
            actual: actual_time_levels,
        });
    }
    let actual_staggering = state.staggering();
    if !specification.staggering.matches(actual_staggering) {
        return Err(ArwModelError::RegistryStaggeringMismatch {
            name: specification.name,
            expected: specification.staggering.description(),
            actual: actual_staggering,
        });
    }
    Ok(())
}

fn registry_field_for(specification: &RequiredState) -> crate::ArwRegistryField {
    use crate::{ArwColumnField, ArwGeopotentialField, ArwMassField, ArwRegistryField};

    match specification.name {
        "ph" | "phb" => ArwRegistryField::Geopotential(match specification.name {
            "ph" => ArwGeopotentialField::CurrentPerturbation,
            _ => ArwGeopotentialField::BaseState,
        }),
        "mu" | "mub" | "rainnc" | "rainncv" => ArwRegistryField::Column(match specification.name {
            "mu" => ArwColumnField::CurrentPerturbationMass,
            "mub" => ArwColumnField::BaseMass,
            "rainnc" => ArwColumnField::AccumulatedPrecipitation,
            _ => ArwColumnField::StepPrecipitation,
        }),
        "u" => ArwRegistryField::Mass(ArwMassField::CurrentWestEastVelocity),
        "v" => ArwRegistryField::Mass(ArwMassField::CurrentSouthNorthVelocity),
        "w" => ArwRegistryField::Mass(ArwMassField::CurrentVerticalVelocity),
        "t" => ArwRegistryField::Mass(ArwMassField::CurrentPotentialTemperature),
        "p" => ArwRegistryField::Mass(ArwMassField::PerturbationPressure),
        "al" => ArwRegistryField::Mass(ArwMassField::PerturbationInverseDensity),
        "pb" => ArwRegistryField::Mass(ArwMassField::BasePressure),
        "alb" => ArwRegistryField::Mass(ArwMassField::BaseInverseDensity),
        "ww" | "ww_m" | "php" | "h_diabatic" | "qv_diabatic" | "qc_diabatic" | "rho"
        | "th_phy_m_t0" => ArwRegistryField::RestartVolume(match specification.name {
            "ww" => crate::ArwRestartVolumeField::VerticalMassFlux,
            "ww_m" => crate::ArwRestartVolumeField::AverageVerticalMassFlux,
            "php" => crate::ArwRestartVolumeField::PressurePointGeopotential,
            "h_diabatic" => crate::ArwRestartVolumeField::DiabaticHeating,
            "qv_diabatic" => crate::ArwRestartVolumeField::WaterVaporDiabaticTendency,
            "qc_diabatic" => crate::ArwRestartVolumeField::CloudWaterDiabaticTendency,
            "rho" => crate::ArwRestartVolumeField::DryAirDensity,
            _ => crate::ArwRestartVolumeField::PerturbationDryPotentialTemperature,
        }),
        "msfux" | "msfuy" | "msfvx" | "msfvx_inv" | "msfvy" | "msftx" | "msfty" | "ht" => {
            ArwRegistryField::Map(match specification.name {
                "msfux" => crate::ArwMapField::WestEastVelocityX,
                "msfuy" => crate::ArwMapField::WestEastVelocityY,
                "msfvx" => crate::ArwMapField::SouthNorthVelocityX,
                "msfvx_inv" => crate::ArwMapField::InverseSouthNorthVelocityX,
                "msfvy" => crate::ArwMapField::SouthNorthVelocityY,
                "msftx" => crate::ArwMapField::MassPointX,
                "msfty" => crate::ArwMapField::MassPointY,
                _ => crate::ArwMapField::TerrainHeight,
            })
        }
        _ => unreachable!("required Registry state table contains only known names"),
    }
}

#[cfg(test)]
mod tests {
    use wrf_registry::{RegistryParser, RuntimeConfigurationChoice};

    use super::*;

    const REGISTRY: &str =
        include_str!("../../../parity/registry-backed-arw-trajectory/Registry.model");

    #[test]
    fn binds_the_verbatim_dependency_closed_registry_projection() {
        let document = RegistryParser::parse("Registry.model", REGISTRY).unwrap();

        let binding = ArwRegistryBinding::try_new(
            &document,
            &[RuntimeConfigurationChoice::new("mp_physics", 1)],
        )
        .unwrap();

        assert_eq!(binding.moisture_layout().members().len(), 3);
    }

    #[test]
    fn rejects_missing_restart_state_before_storage_allocation() {
        let source = REGISTRY
            .lines()
            .filter(|line| line.split_whitespace().nth(2) != Some("h_diabatic"))
            .collect::<Vec<_>>()
            .join("\n");
        let document = RegistryParser::parse("Registry.missing", &source).unwrap();

        assert!(matches!(
            ArwRegistryBinding::try_new(
                &document,
                &[RuntimeConfigurationChoice::new("mp_physics", 1)]
            ),
            Err(ArwModelError::MissingRegistryState { name: "h_diabatic" })
        ));
    }

    #[test]
    fn rejects_wrong_dimension_order_and_extra_time_levels() {
        let wrong_order = REGISTRY.replacen("state real u ikj", "state real u ijk", 1);
        let document = RegistryParser::parse("Registry.order", &wrong_order).unwrap();
        assert!(matches!(
            ArwRegistryBinding::try_new(
                &document,
                &[RuntimeConfigurationChoice::new("mp_physics", 1)]
            ),
            Err(ArwModelError::RegistryDimensionsMismatch { name: "u", .. })
        ));

        let extra_level =
            REGISTRY.replacen("state real u ikj dyn_em 2", "state real u ikj dyn_em 3", 1);
        let document = RegistryParser::parse("Registry.level", &extra_level).unwrap();
        assert!(matches!(
            ArwRegistryBinding::try_new(
                &document,
                &[RuntimeConfigurationChoice::new("mp_physics", 1)]
            ),
            Err(ArwModelError::RegistryTimeLevelMismatch {
                field: crate::ArwRegistryField::Mass(crate::ArwMassField::CurrentWestEastVelocity),
                expected: 2,
                actual: 3
            })
        ));
    }
}
