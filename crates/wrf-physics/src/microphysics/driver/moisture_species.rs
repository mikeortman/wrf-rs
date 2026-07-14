use std::fmt;

/// Moisture species role carried in WRF's Registry-ordered `moist` array.
///
/// WRF's generated `P_QV`, `P_QC`, and `P_QR` constants name one-based
/// positions inside the four-dimensional `moist` state. The Rust port keeps
/// the same roles and resolves their positions through a
/// [`crate::MoistureSpeciesPackage`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MoistureSpecies {
    /// Water-vapor mixing ratio (`qv`).
    WaterVapor,
    /// Cloud-water mixing ratio (`qc`).
    CloudWater,
    /// Rain-water mixing ratio (`qr`).
    RainWater,
}

impl MoistureSpecies {
    pub(crate) fn from_registry_name(name: &str) -> Option<Self> {
        match name {
            "qv" => Some(Self::WaterVapor),
            "qc" => Some(Self::CloudWater),
            "qr" => Some(Self::RainWater),
            _ => None,
        }
    }

    /// Returns the Registry state name associated with this role.
    pub const fn registry_name(self) -> &'static str {
        match self {
            Self::WaterVapor => "qv",
            Self::CloudWater => "qc",
            Self::RainWater => "qr",
        }
    }
}

impl fmt::Display for MoistureSpecies {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WaterVapor => formatter.write_str("water vapor (qv)"),
            Self::CloudWater => formatter.write_str("cloud water (qc)"),
            Self::RainWater => formatter.write_str("rain water (qr)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_match_registry_state_entries() {
        assert_eq!(MoistureSpecies::WaterVapor.registry_name(), "qv");
        assert_eq!(MoistureSpecies::CloudWater.registry_name(), "qc");
        assert_eq!(MoistureSpecies::RainWater.registry_name(), "qr");
        assert_eq!(
            MoistureSpecies::from_registry_name("qc"),
            Some(MoistureSpecies::CloudWater)
        );
        assert_eq!(MoistureSpecies::from_registry_name("qi"), None);
    }

    #[test]
    fn display_names_role_and_registry_state() {
        assert_eq!(MoistureSpecies::WaterVapor.to_string(), "water vapor (qv)");
    }
}
