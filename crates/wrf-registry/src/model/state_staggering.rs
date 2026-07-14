/// Staggering and feedback flags parsed from a state entry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StateStaggering {
    pub(crate) x: bool,
    pub(crate) y: bool,
    pub(crate) z: bool,
    pub(crate) nmm_vertical_grid: bool,
    pub(crate) microphysics_variable: bool,
    pub(crate) full_feedback: bool,
    pub(crate) no_feedback: bool,
}

impl StateStaggering {
    /// Returns whether the state is staggered along X.
    #[must_use]
    pub const fn is_x_staggered(self) -> bool {
        self.x
    }

    /// Returns whether the state is staggered along Y.
    #[must_use]
    pub const fn is_y_staggered(self) -> bool {
        self.y
    }

    /// Returns whether the state is staggered along Z.
    #[must_use]
    pub const fn is_z_staggered(self) -> bool {
        self.z
    }

    /// Returns whether the state uses WRF's NMM vertical grid flag.
    #[must_use]
    pub const fn uses_nmm_vertical_grid(self) -> bool {
        self.nmm_vertical_grid
    }

    /// Returns whether the state is marked as a microphysics variable.
    #[must_use]
    pub const fn is_microphysics_variable(self) -> bool {
        self.microphysics_variable
    }

    /// Returns whether full nest feedback is enabled.
    #[must_use]
    pub const fn has_full_feedback(self) -> bool {
        self.full_feedback
    }

    /// Returns whether nest feedback is disabled.
    #[must_use]
    pub const fn has_no_feedback(self) -> bool {
        self.no_feedback
    }
}
