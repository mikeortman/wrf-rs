/// Specified or nested-domain acoustic relaxation-zone behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticRelaxationZone {
    /// WRF's ordinary non-specified, non-nested bounds.
    Disabled,
    /// WRF `specified .or. nested` bounds with the supplied `spec_zone` width.
    Active {
        /// Number of mass points excluded at each nonperiodic physical edge.
        width: usize,
    },
}

impl AcousticRelaxationZone {
    pub(crate) const fn width(self) -> Option<usize> {
        match self {
            Self::Disabled => None,
            Self::Active { width } => Some(width),
        }
    }
}
