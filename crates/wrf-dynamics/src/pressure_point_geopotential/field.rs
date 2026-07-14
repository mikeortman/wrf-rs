use std::fmt;

/// Field role used by pressure-point geopotential shape validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PressurePointGeopotentialField {
    /// Pressure-point full-geopotential output (`php`).
    PressurePointOutput,
    /// Perturbation geopotential on full vertical levels (`ph`).
    PerturbationFullLevel,
    /// Base-state geopotential on full vertical levels (`phb`).
    BaseStateFullLevel,
}

impl fmt::Display for PressurePointGeopotentialField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PressurePointOutput => formatter.write_str("pressure-point geopotential output"),
            Self::PerturbationFullLevel => {
                formatter.write_str("full-level perturbation geopotential")
            }
            Self::BaseStateFullLevel => formatter.write_str("full-level base-state geopotential"),
        }
    }
}
