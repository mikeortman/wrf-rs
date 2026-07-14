/// Lateral-boundary behavior applied while clipping microphysics tiles.
///
/// Mirrors the pinned driver preamble: the specified-boundary zone is skipped
/// only when the domain uses specified (or nested) boundaries, and a channel
/// configuration keeps the full west-east extent active.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MicrophysicsBoundaryPolicy {
    specified: bool,
    channel_switch: bool,
    specified_zone_width: usize,
}

impl MicrophysicsBoundaryPolicy {
    /// Creates the policy from the driver's `specified`, `channel_switch`,
    /// and `spec_zone` inputs.
    pub const fn new(specified: bool, channel_switch: bool, specified_zone_width: usize) -> Self {
        Self {
            specified,
            channel_switch,
            specified_zone_width,
        }
    }

    /// Returns the policy used by idealized cases without specified boundaries.
    pub const fn open() -> Self {
        Self::new(false, false, 0)
    }

    /// Returns the boundary-zone width skipped on clipped axes.
    ///
    /// The pinned driver uses `spec_zone` only when `specified` is set.
    pub(crate) const fn effective_zone_width(self) -> usize {
        if self.specified {
            self.specified_zone_width
        } else {
            0
        }
    }

    /// Returns whether the west-east axis keeps its full extent.
    pub(crate) const fn is_channel(self) -> bool {
        self.channel_switch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_width_applies_only_with_specified_boundaries() {
        assert_eq!(
            MicrophysicsBoundaryPolicy::new(true, false, 2).effective_zone_width(),
            2
        );
        assert_eq!(
            MicrophysicsBoundaryPolicy::new(false, false, 2).effective_zone_width(),
            0
        );
        assert_eq!(MicrophysicsBoundaryPolicy::open().effective_zone_width(), 0);
    }

    #[test]
    fn channel_switch_is_reported_independently_of_zone_width() {
        assert!(MicrophysicsBoundaryPolicy::new(true, true, 1).is_channel());
        assert!(!MicrophysicsBoundaryPolicy::new(true, false, 1).is_channel());
    }
}
