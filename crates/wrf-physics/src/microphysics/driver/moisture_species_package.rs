use crate::{
    MicrophysicsDriverError, MicrophysicsDriverResult, MoistureSpecies, MoistureSpeciesIndex,
};

/// Ordered moisture species associated with one Registry scheme package.
///
/// Mirrors Registry package lines such as
/// `package kesslerscheme mp_physics==1 - moist:qv,qc,qr`, whose species order
/// defines the generated `P_QV`/`P_QC`/`P_QR` positions inside `moist`.
/// Registry-parsed construction is follow-up work owned by the Registry area;
/// until then the pinned orderings are built explicitly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoistureSpeciesPackage {
    species: Vec<MoistureSpecies>,
}

impl MoistureSpeciesPackage {
    /// Creates a package from an explicit species ordering.
    ///
    /// # Errors
    ///
    /// Returns an error if the ordering is empty or names a species twice,
    /// which would make the generated index semantics ambiguous.
    pub fn try_new(species: Vec<MoistureSpecies>) -> MicrophysicsDriverResult<Self> {
        if species.is_empty() {
            return Err(MicrophysicsDriverError::EmptyMoisturePackage);
        }
        for (position, candidate) in species.iter().enumerate() {
            if species[..position].contains(candidate) {
                return Err(MicrophysicsDriverError::DuplicateMoistureSpecies {
                    species: *candidate,
                });
            }
        }
        Ok(Self { species })
    }

    /// Creates the pinned `kesslerscheme` package ordering `moist:qv,qc,qr`.
    pub fn kessler() -> Self {
        Self {
            species: vec![
                MoistureSpecies::WaterVapor,
                MoistureSpecies::CloudWater,
                MoistureSpecies::RainWater,
            ],
        }
    }

    /// Returns the number of species carried by the package.
    pub fn species_count(&self) -> usize {
        self.species.len()
    }

    /// Returns the ordered species roles.
    pub fn species(&self) -> &[MoistureSpecies] {
        &self.species
    }

    /// Returns the zero-based position of one species, if present.
    pub fn index_of(&self, species: MoistureSpecies) -> Option<MoistureSpeciesIndex> {
        self.species
            .iter()
            .position(|candidate| *candidate == species)
            .map(MoistureSpeciesIndex::new)
    }

    /// Returns the position of a species that a scheme requires.
    ///
    /// # Errors
    ///
    /// Returns an error naming the missing species, mirroring the driver's
    /// `arguments not present for calling kessler` fatal rejection.
    pub fn require_index_of(
        &self,
        species: MoistureSpecies,
    ) -> MicrophysicsDriverResult<MoistureSpeciesIndex> {
        self.index_of(species)
            .ok_or(MicrophysicsDriverError::MissingMoistureSpecies { species })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kessler_package_orders_qv_qc_qr() {
        let package = MoistureSpeciesPackage::kessler();

        assert_eq!(
            package.species(),
            [
                MoistureSpecies::WaterVapor,
                MoistureSpecies::CloudWater,
                MoistureSpecies::RainWater,
            ]
        );
        assert_eq!(
            package.index_of(MoistureSpecies::RainWater),
            Some(MoistureSpeciesIndex::new(2))
        );
    }

    #[test]
    fn try_new_rejects_empty_and_duplicate_orderings() {
        assert_eq!(
            MoistureSpeciesPackage::try_new(Vec::new()),
            Err(MicrophysicsDriverError::EmptyMoisturePackage)
        );
        assert_eq!(
            MoistureSpeciesPackage::try_new(vec![
                MoistureSpecies::WaterVapor,
                MoistureSpecies::WaterVapor,
            ]),
            Err(MicrophysicsDriverError::DuplicateMoistureSpecies {
                species: MoistureSpecies::WaterVapor,
            })
        );
    }

    #[test]
    fn require_index_of_names_the_missing_species() {
        let package = MoistureSpeciesPackage::try_new(vec![MoistureSpecies::WaterVapor]).unwrap();

        assert_eq!(
            package.require_index_of(MoistureSpecies::RainWater),
            Err(MicrophysicsDriverError::MissingMoistureSpecies {
                species: MoistureSpecies::RainWater,
            })
        );
    }
}
