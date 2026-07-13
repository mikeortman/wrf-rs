use wrf_compute::FieldStorage;

use crate::{HeldSuarezDampingFields, HeldSuarezDampingRegion, HeldSuarezDampingResult};

/// Backend capability for Held-Suarez idealized momentum damping.
pub trait HeldSuarezDampingKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Applies pressure-dependent Rayleigh damping to momentum tendencies.
    ///
    /// # Errors
    ///
    /// Returns an error if any field shape differs from the region or CPU
    /// execution fails.
    fn apply_held_suarez_damping(
        &self,
        fields: HeldSuarezDampingFields<'_, Self::Field>,
        region: &HeldSuarezDampingRegion,
    ) -> HeldSuarezDampingResult<()>;
}
