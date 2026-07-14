/// Active WRF moisture species used in momentum coefficients.
///
/// WRF Registry generation reserves Fortran scalar slot 1 and defines
/// `PARAM_FIRST_SCALAR = 2`. This view contains only the active fields from
/// slots 2 through `n_moist`; an empty slice selects WRF's dry defaults.
pub struct MoistureSpecies<'a, Field> {
    pub(crate) active: &'a [Field],
}

impl<Field> Copy for MoistureSpecies<'_, Field> {}

impl<Field> Clone for MoistureSpecies<'_, Field> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> MoistureSpecies<'a, Field> {
    /// Borrows active species in their exact WRF accumulation order.
    pub const fn new(active: &'a [Field]) -> Self {
        Self { active }
    }

    /// Returns the number of active moisture species.
    pub const fn active_species_count(self) -> usize {
        self.active.len()
    }

    /// Returns whether WRF's no-active-species defaults apply.
    pub const fn is_empty(self) -> bool {
        self.active.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_active_species_without_wrf_padding_slot() {
        let fields = [1_u8, 2_u8, 3_u8];
        let moisture = MoistureSpecies::new(&fields);

        assert_eq!(moisture.active_species_count(), 3);
        assert!(!moisture.is_empty());
        assert!(MoistureSpecies::<u8>::new(&[]).is_empty());
    }
}
