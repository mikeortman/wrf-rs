use std::fmt;

use crate::{WrfIoError, WrfIoResult};

/// A validated Registry-declared NetCDF dimension name.
///
/// Instances are only created through [`crate::WrfDimensionName::try_from_name`],
/// which routes the fixed ARW core names to their dedicated variants, so a
/// Registry name is always outside that core set and always valid NetCDF
/// syntax.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WrfRegistryDimensionName(String);

impl WrfRegistryDimensionName {
    pub(crate) fn try_new(value: impl Into<String>) -> WrfIoResult<Self> {
        let value = value.into();
        if netcdf3::is_valid_name(&value) {
            return Ok(Self(value));
        }
        Err(WrfIoError::InvalidDimensionName { value })
    }

    /// Returns the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WrfRegistryDimensionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_rejects_invalid_netcdf_names() {
        assert!(matches!(
            WrfRegistryDimensionName::try_new("bad/name"),
            Err(WrfIoError::InvalidDimensionName { .. })
        ));
        assert_eq!(
            WrfRegistryDimensionName::try_new("soil_layers_stag")
                .unwrap()
                .as_str(),
            "soil_layers_stag"
        );
    }
}
