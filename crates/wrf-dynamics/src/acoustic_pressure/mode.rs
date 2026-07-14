/// Governing-equation mode selected by WRF's `non_hydrostatic` flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticPressureMode {
    /// Diagnose perturbation inverse density and pressure without changing geopotential.
    Nonhydrostatic,
    /// Diagnose pressure and inverse density, then integrate hydrostatic geopotential.
    Hydrostatic,
}
