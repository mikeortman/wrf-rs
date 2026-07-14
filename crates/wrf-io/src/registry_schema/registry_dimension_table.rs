use crate::{WrfDimension, WrfDimensionName, WrfIoError, WrfIoResult};

const DATE_STRING_LENGTH: usize = 19;
const MAXIMUM_DIMENSION_SLOTS: usize = 2_000;

/// First-use dimension registration matching WRF's NetCDF writer table.
///
/// `ext_ncd_open_for_write_begin` defines the unlimited `Time` dimension and
/// then seeds a slot table whose first entry is `DateStrLen`; every later
/// dimension occupies the next slot in first-use order. A named dimension is
/// reused by name and must keep its length. An anonymous dimension (constant
/// coordinate axis or missing `dimspec` data name) reuses the first slot with
/// the same length regardless of its name, and otherwise claims a new slot
/// named `DIMnnnn` after the slot's one-based index.
#[derive(Debug)]
pub(crate) struct RegistryDimensionTable {
    slots: Vec<RegistryDimensionSlot>,
}

#[derive(Debug)]
struct RegistryDimensionSlot {
    name: WrfDimensionName,
    length: usize,
}

impl RegistryDimensionTable {
    /// Creates the table with WRF's pre-seeded `DateStrLen` slot.
    pub(crate) fn new() -> Self {
        Self {
            slots: vec![RegistryDimensionSlot {
                name: WrfDimensionName::DateStringLength,
                length: DATE_STRING_LENGTH,
            }],
        }
    }

    /// Registers a named dimension, enforcing one length per name.
    pub(crate) fn require_named(
        &mut self,
        name: WrfDimensionName,
        length: usize,
    ) -> WrfIoResult<()> {
        if name == WrfDimensionName::Time {
            return Err(WrfIoError::ReservedRegistryDimensionName {
                dimension: name.as_str().to_owned(),
            });
        }
        if let Some(slot) = self.slots.iter().find(|slot| slot.name == name) {
            if slot.length == length {
                return Ok(());
            }
            return Err(WrfIoError::DimensionLengthConflict {
                dimension: name.as_str().to_owned(),
                existing: slot.length,
                requested: length,
            });
        }
        if Self::placeholder_index(&name).is_some_and(|index| index > self.slots.len()) {
            return Err(WrfIoError::ReservedRegistryDimensionName {
                dimension: name.as_str().to_owned(),
            });
        }

        self.require_available_slot()?;
        self.slots.push(RegistryDimensionSlot { name, length });
        Ok(())
    }

    /// Registers an anonymous dimension, reusing any slot of equal length.
    pub(crate) fn require_anonymous(&mut self, length: usize) -> WrfIoResult<WrfDimensionName> {
        if let Some(slot) = self.slots.iter().find(|slot| slot.length == length) {
            return Ok(slot.name.clone());
        }

        self.require_available_slot()?;
        let name = WrfDimensionName::try_from_name(&format!("DIM{:04}", self.slots.len() + 1))?;
        self.slots.push(RegistryDimensionSlot {
            name: name.clone(),
            length,
        });
        Ok(name)
    }

    fn placeholder_index(name: &WrfDimensionName) -> Option<usize> {
        let digits = name.as_str().strip_prefix("DIM")?;
        if digits.len() != 4 || !digits.bytes().all(|digit| digit.is_ascii_digit()) {
            return None;
        }
        let index = digits.parse::<usize>().ok()?;
        (2..=MAXIMUM_DIMENSION_SLOTS)
            .contains(&index)
            .then_some(index)
    }

    fn require_available_slot(&self) -> WrfIoResult<()> {
        if self.slots.len() < MAXIMUM_DIMENSION_SLOTS {
            return Ok(());
        }
        Err(WrfIoError::RegistryDimensionTableFull {
            maximum: MAXIMUM_DIMENSION_SLOTS,
        })
    }

    /// Returns file-order dimensions: unlimited `Time`, then first-use slots.
    pub(crate) fn into_dimensions(self, time_length: usize) -> Vec<WrfDimension> {
        let mut dimensions = Vec::with_capacity(self.slots.len() + 1);
        dimensions.push(WrfDimension::unlimited(WrfDimensionName::Time, time_length));
        dimensions.extend(
            self.slots
                .into_iter()
                .map(|slot| WrfDimension::fixed(slot.name, slot.length)),
        );
        dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_dimensions_reuse_by_name_and_reject_length_conflicts() {
        let mut table = RegistryDimensionTable::new();
        let west_east = WrfDimensionName::WestEast;
        table.require_named(west_east.clone(), 4).unwrap();
        table.require_named(west_east.clone(), 4).unwrap();
        assert!(matches!(
            table.require_named(west_east, 5),
            Err(WrfIoError::DimensionLengthConflict {
                existing: 4,
                requested: 5,
                ..
            })
        ));
    }

    #[test]
    fn anonymous_dimensions_reuse_equal_lengths_and_take_slot_names() {
        let mut table = RegistryDimensionTable::new();
        table.require_named(WrfDimensionName::WestEast, 4).unwrap();

        let reused = table.require_anonymous(4).unwrap();
        assert_eq!(reused, WrfDimensionName::WestEast);
        let reused_date_string = table.require_anonymous(19).unwrap();
        assert_eq!(reused_date_string, WrfDimensionName::DateStringLength);

        let fresh = table.require_anonymous(7).unwrap();
        assert_eq!(fresh.as_str(), "DIM0003");

        let dimensions = table.into_dimensions(1);
        assert_eq!(dimensions.len(), 4);
        assert!(dimensions[0].is_unlimited());
        assert_eq!(dimensions[3].name().as_str(), "DIM0003");
        assert_eq!(dimensions[3].length(), 7);
    }

    #[test]
    fn named_dimensions_respect_unused_wrf_placeholder_slots() {
        let mut table = RegistryDimensionTable::new();
        let placeholder = WrfDimensionName::try_from_name("DIM0002").unwrap();
        assert!(matches!(
            table.require_named(placeholder.clone(), 2),
            Err(WrfIoError::ReservedRegistryDimensionName { .. })
        ));

        table.require_named(WrfDimensionName::WestEast, 4).unwrap();
        table.require_named(placeholder, 2).unwrap();
        let dimensions = table.into_dimensions(1);
        assert_eq!(dimensions[3].name().as_str(), "DIM0002");
        assert_eq!(dimensions[3].length(), 2);
    }

    #[test]
    fn dimension_table_rejects_a_slot_beyond_wrf_maximum() {
        let mut table = RegistryDimensionTable::new();
        for index in 2..=MAXIMUM_DIMENSION_SLOTS {
            let name =
                WrfDimensionName::try_from_name(&format!("named_dimension_{index}")).unwrap();
            table.require_named(name, 10_000 + index).unwrap();
        }

        assert!(matches!(
            table.require_anonymous(usize::MAX),
            Err(WrfIoError::RegistryDimensionTableFull {
                maximum: MAXIMUM_DIMENSION_SLOTS
            })
        ));
    }
}
