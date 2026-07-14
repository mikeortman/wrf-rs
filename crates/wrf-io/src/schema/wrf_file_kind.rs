/// Identifies the WRF role of a dataset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrfFileKind {
    /// Initial state consumed at the beginning of an integration.
    Initialization,
    /// Checkpoint state that must resume the same trajectory.
    Restart,
}

impl WrfFileKind {
    pub(crate) const fn requires_restart_flag(self) -> bool {
        matches!(self, Self::Restart)
    }
}
