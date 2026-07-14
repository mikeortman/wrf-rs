/// One signed runtime-configuration value used to select Registry packages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfigurationChoice {
    name: String,
    value: i32,
}

impl RuntimeConfigurationChoice {
    /// Creates one domain's value for a named Registry runtime configuration.
    #[must_use]
    pub fn new(name: impl Into<String>, value: i32) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Returns the Registry runtime-configuration symbol.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the signed value used for package equality checks.
    #[must_use]
    pub const fn value(&self) -> i32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_signed_runtime_configuration_values() {
        let choice = RuntimeConfigurationChoice::new("mp_physics", -9);

        assert_eq!(choice.name(), "mp_physics");
        assert_eq!(choice.value(), -9);
    }
}
