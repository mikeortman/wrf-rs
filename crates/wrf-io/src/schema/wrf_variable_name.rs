use std::fmt;

use crate::{WrfIoError, WrfIoResult};

/// A validated NetCDF variable name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WrfVariableName(String);

impl WrfVariableName {
    /// Validates a NetCDF-compatible variable name.
    pub fn try_new(value: impl Into<String>) -> WrfIoResult<Self> {
        let value = value.into();
        if netcdf3::is_valid_name(&value) {
            return Ok(Self(value));
        }

        Err(WrfIoError::InvalidVariableName { value })
    }

    /// Returns the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WrfVariableName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_uses_netcdf_name_rules() {
        assert!(WrfVariableName::try_new("").is_err());
        assert!(WrfVariableName::try_new("contains/slash").is_err());
        assert_eq!(WrfVariableName::try_new("1T").unwrap().as_str(), "1T");
        assert_eq!(
            WrfVariableName::try_new("QVAPOR").unwrap().as_str(),
            "QVAPOR"
        );
    }
}
