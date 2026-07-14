use crate::{WrfIoError, WrfIoResult};

/// Mass-grid dimensions for a minimum ARW state file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WrfGridDimensions {
    west_east: usize,
    south_north: usize,
    bottom_top: usize,
}

impl WrfGridDimensions {
    /// Creates checked mass-grid dimensions and reserves their staggered peers.
    pub fn try_new(west_east: usize, south_north: usize, bottom_top: usize) -> WrfIoResult<Self> {
        for (name, length) in [
            ("west_east", west_east),
            ("south_north", south_north),
            ("bottom_top", bottom_top),
        ] {
            if length == 0 {
                return Err(WrfIoError::EmptyDimension { name });
            }
            length
                .checked_add(1)
                .ok_or(WrfIoError::DimensionLengthOverflow { name, length })?;
        }

        Ok(Self {
            west_east,
            south_north,
            bottom_top,
        })
    }

    /// Returns the mass-grid west-east point count.
    pub const fn west_east(self) -> usize {
        self.west_east
    }

    /// Returns the mass-grid south-north point count.
    pub const fn south_north(self) -> usize {
        self.south_north
    }

    /// Returns the mass-grid vertical level count.
    pub const fn bottom_top(self) -> usize {
        self.bottom_top
    }

    pub(crate) const fn west_east_staggered(self) -> usize {
        self.west_east + 1
    }

    pub(crate) const fn south_north_staggered(self) -> usize {
        self.south_north + 1
    }

    pub(crate) const fn bottom_top_staggered(self) -> usize {
        self.bottom_top + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_rejects_empty_and_unstaggerable_dimensions() {
        assert!(matches!(
            WrfGridDimensions::try_new(0, 3, 2),
            Err(WrfIoError::EmptyDimension { name: "west_east" })
        ));
        assert!(matches!(
            WrfGridDimensions::try_new(4, usize::MAX, 2),
            Err(WrfIoError::DimensionLengthOverflow {
                name: "south_north",
                ..
            })
        ));
    }
}
