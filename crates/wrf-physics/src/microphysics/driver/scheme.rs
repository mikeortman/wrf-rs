use crate::{MicrophysicsDriverError, MicrophysicsDriverResult};

/// Microphysics scheme selected by WRF's `mp_physics` namelist option.
///
/// Only the schemes ported so far are representable; every other upstream
/// `mp_physics` value is rejected as a typed error instead of silently
/// falling through the driver dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicrophysicsScheme {
    /// `mp_physics == 0`: the driver returns without touching any field.
    Disabled,
    /// `mp_physics == 1`: the Kessler warm-rain scheme (`KESSLERSCHEME`).
    Kessler,
}

impl MicrophysicsScheme {
    /// Maps a namelist `mp_physics` value onto a ported scheme.
    ///
    /// # Errors
    ///
    /// Returns an error carrying the unsupported `mp_physics` value for every
    /// upstream scheme that has not been ported.
    pub fn try_from_mp_physics(mp_physics: i32) -> MicrophysicsDriverResult<Self> {
        match mp_physics {
            0 => Ok(Self::Disabled),
            1 => Ok(Self::Kessler),
            unsupported => Err(MicrophysicsDriverError::UnsupportedScheme {
                mp_physics: unsupported,
            }),
        }
    }

    /// Returns the upstream `mp_physics` namelist value for this scheme.
    pub const fn mp_physics(self) -> i32 {
        match self {
            Self::Disabled => 0,
            Self::Kessler => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_namelist_values_and_round_trips() {
        assert_eq!(
            MicrophysicsScheme::try_from_mp_physics(0),
            Ok(MicrophysicsScheme::Disabled)
        );
        assert_eq!(
            MicrophysicsScheme::try_from_mp_physics(1),
            Ok(MicrophysicsScheme::Kessler)
        );
        assert_eq!(MicrophysicsScheme::Kessler.mp_physics(), 1);
    }

    #[test]
    fn rejects_unported_schemes_with_the_offending_value() {
        assert_eq!(
            MicrophysicsScheme::try_from_mp_physics(6),
            Err(MicrophysicsDriverError::UnsupportedScheme { mp_physics: 6 })
        );
    }
}
