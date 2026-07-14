use std::fmt;

use crate::{WrfIoError, WrfIoResult};

/// A WRF timestamp in `YYYY-MM-DD_HH:MM:SS` form.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WrfTimestamp(String);

impl WrfTimestamp {
    /// Validates and stores one fixed-width WRF timestamp.
    pub fn try_new(value: impl Into<String>) -> WrfIoResult<Self> {
        let value = value.into();
        if Self::has_valid_shape(&value) {
            return Ok(Self(value));
        }

        Err(WrfIoError::InvalidTimestamp { value })
    }

    /// Returns the validated timestamp text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn has_valid_shape(value: &str) -> bool {
        let bytes = value.as_bytes();
        if bytes.len() != 19 {
            return false;
        }

        const SEPARATORS: [(usize, u8); 5] =
            [(4, b'-'), (7, b'-'), (10, b'_'), (13, b':'), (16, b':')];
        if SEPARATORS
            .iter()
            .any(|&(index, expected)| bytes[index] != expected)
        {
            return false;
        }

        bytes.iter().enumerate().all(|(index, byte)| {
            SEPARATORS.iter().any(|&(separator, _)| separator == index) || byte.is_ascii_digit()
        })
    }
}

impl fmt::Display for WrfTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_accepts_the_fixed_wrf_timestamp_shape() {
        let timestamp = WrfTimestamp::try_new("2000-09-18_16:42:01").unwrap();

        assert_eq!(timestamp.as_str(), "2000-09-18_16:42:01");
    }

    #[test]
    fn try_new_rejects_wrong_width_separators_and_non_digits() {
        for invalid in [
            "2000-09-18 16:42:01",
            "2000-09-18_16:42",
            "2000-09-18_16:4x:01",
        ] {
            assert!(matches!(
                WrfTimestamp::try_new(invalid),
                Err(WrfIoError::InvalidTimestamp { .. })
            ));
        }
    }
}
