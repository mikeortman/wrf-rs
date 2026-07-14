use crate::{WrfIoResult, WrfVariableName, WrfVariableValues};

/// One named borrowed variable supplied to a WRF writer.
#[derive(Clone, Debug)]
pub struct WrfVariableView<'a> {
    name: WrfVariableName,
    values: WrfVariableValues<'a>,
}

impl<'a> WrfVariableView<'a> {
    /// Creates a view with a validated variable name.
    pub fn try_new(name: impl Into<String>, values: WrfVariableValues<'a>) -> WrfIoResult<Self> {
        Ok(Self {
            name: WrfVariableName::try_new(name)?,
            values,
        })
    }

    /// Returns the variable name.
    pub const fn name(&self) -> &WrfVariableName {
        &self.name
    }

    /// Returns the borrowed values.
    pub const fn values(&self) -> WrfVariableValues<'a> {
        self.values
    }
}
