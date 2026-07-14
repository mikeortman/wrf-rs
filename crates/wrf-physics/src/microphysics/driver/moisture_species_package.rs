use crate::{
    MicrophysicsDriverError, MicrophysicsDriverResult, MoistureSpecies, MoistureSpeciesIndex,
};
use wrf_registry::ResolvedScalarArrayLayout;

/// Ordered moisture species associated with one Registry scheme package.
///
/// Mirrors Registry package lines such as
/// `package kesslerscheme mp_physics==1 - moist:qv,qc,qr`, whose species order
/// defines the generated `P_QV`/`P_QC`/`P_QR` positions inside `moist`.
/// Generic Registry parsing and index resolution remain Registry-owned; this
/// physics type owns the conversion of resolved `moist` names into species.
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

    /// Converts a generic resolved Registry `moist` layout into physics roles.
    ///
    /// Members retain the zero-based dense order established by Registry
    /// package resolution. Definition-table and one-based packed indices stay
    /// Registry-owned and are not reinterpreted by the numerical driver.
    ///
    /// # Errors
    ///
    /// Returns a typed error if `layout` is not the `moist` scalar array, a
    /// member has no supported moisture role, or the resulting package is
    /// empty or contains a duplicate role.
    pub fn try_from_registry_layout(
        layout: &ResolvedScalarArrayLayout,
    ) -> MicrophysicsDriverResult<Self> {
        if layout.scalar_array_name() != "moist" {
            return Err(MicrophysicsDriverError::UnexpectedMoistureScalarArray {
                actual: layout.scalar_array_name().to_owned(),
            });
        }
        let species = layout
            .members()
            .iter()
            .map(|member| {
                MoistureSpecies::from_registry_name(member.name()).ok_or_else(|| {
                    MicrophysicsDriverError::UnsupportedMoistureSpecies {
                        name: member.name().to_owned(),
                    }
                })
            })
            .collect::<MicrophysicsDriverResult<Vec<_>>>()?;
        Self::try_new(species)
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
    use wrf_registry::{RegistryParser, RuntimeConfigurationChoice};

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

    #[test]
    fn registry_layout_conversion_preserves_canonical_and_reordered_dense_order() {
        let source = "\
dimspec i 1 standard_domain x west_east
dimspec k 2 standard_domain z bottom_top
dimspec j 3 standard_domain y south_north
rconfig integer mp_physics namelist,physics 1 -1 - mp_physics \"\" \"\"
state real - ikjftb moist 1 - - - - -
state real qv ikjftb moist 1 - - QVAPOR vapor 1
state real qc ikjftb moist 1 - - QCLOUD cloud 1
state real qr ikjftb moist 1 - - QRAIN rain 1
package canonical mp_physics==1 - moist:qv,qc,qr
package reordered mp_physics==2 - moist:qr,qv,qc
";
        let document = RegistryParser::parse("Registry.physics", source).unwrap();

        for (choice, expected) in [
            (
                1,
                vec![
                    MoistureSpecies::WaterVapor,
                    MoistureSpecies::CloudWater,
                    MoistureSpecies::RainWater,
                ],
            ),
            (
                2,
                vec![
                    MoistureSpecies::RainWater,
                    MoistureSpecies::WaterVapor,
                    MoistureSpecies::CloudWater,
                ],
            ),
        ] {
            let layouts = document
                .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new(
                    "mp_physics",
                    choice,
                )])
                .unwrap();
            let package = MoistureSpeciesPackage::try_from_registry_layout(&layouts[0]).unwrap();

            assert_eq!(package.species(), expected);
            for (dense_index, species) in expected.into_iter().enumerate() {
                assert_eq!(
                    package.index_of(species),
                    Some(MoistureSpeciesIndex::new(dense_index))
                );
            }
        }
    }

    #[test]
    fn registry_layout_conversion_rejects_wrong_array_and_unknown_species() {
        let sources = [
            (
                "scalar",
                "state real - if scalar 1 - - - - -\nstate real qv if scalar 1 - - QV vapor 1\npackage selected option==1 - scalar:qv\n",
                MicrophysicsDriverError::UnexpectedMoistureScalarArray {
                    actual: "scalar".to_owned(),
                },
            ),
            (
                "moist",
                "state real - if moist 1 - - - - -\nstate real qi if moist 1 - - QI ice 1\npackage selected option==1 - moist:qi\n",
                MicrophysicsDriverError::UnsupportedMoistureSpecies {
                    name: "qi".to_owned(),
                },
            ),
        ];

        for (array_name, body, expected) in sources {
            let source = format!(
                "dimspec i 1 standard_domain x west_east\nrconfig integer option namelist,test 1 0 - option \"\" \"\"\n{body}"
            );
            let document = RegistryParser::parse("Registry.physics-error", &source).unwrap();
            let layouts = document
                .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("option", 1)])
                .unwrap();
            assert_eq!(layouts[0].scalar_array_name(), array_name);
            assert_eq!(
                MoistureSpeciesPackage::try_from_registry_layout(&layouts[0]),
                Err(expected)
            );
        }
    }
}
