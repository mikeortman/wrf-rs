use wrf_compute::{FieldStorage, GridShape};

use crate::{
    KesslerMicrophysicsField, KesslerMicrophysicsFields, KesslerMicrophysicsKernels,
    KesslerMicrophysicsParameters, KesslerMicrophysicsRegion, MicrophysicsDriverDomain,
    MicrophysicsDriverError, MicrophysicsDriverFields, MicrophysicsDriverResult,
    MicrophysicsDriverWorkspace, MicrophysicsScheme, MicrophysicsTile, MoistureSpecies,
    MoistureSpeciesPackage,
};

/// Scheme-specific configuration bound to one driver.
#[derive(Clone, Debug)]
enum MicrophysicsDispatch {
    Disabled,
    Kessler(KesslerDispatch),
}

/// Registry package and resolved positions required by Kessler dispatch.
#[derive(Clone, Debug)]
struct KesslerDispatch {
    moisture_package: MoistureSpeciesPackage,
    species_selection: KesslerSpeciesSelection,
}

/// Positions of the three Kessler species inside the ordered moisture slice.
#[derive(Clone, Copy, Debug)]
struct KesslerSpeciesSelection {
    water_vapor: usize,
    cloud_water: usize,
    rain_water: usize,
}

/// Typed port of WRF's microphysics driver dispatch for ported schemes.
///
/// Owns scheme selection, Registry-ordered moisture-species resolution, and
/// the pinned per-tile boundary clipping, then dispatches each active tile to
/// the scheme kernel. Per-step precipitation (`RAINNCV`) and accumulated
/// precipitation (`RAINNC`) are produced by the kernel inside each dispatch,
/// so repeated driver calls accumulate restart-relevant precipitation exactly
/// like repeated upstream timesteps.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_physics::{
///     KesslerMicrophysicsParameters, MicrophysicsBoundaryPolicy, MicrophysicsDriver,
///     MicrophysicsDriverDomain, MicrophysicsDriverFields, MicrophysicsTile,
///     MoistureSpeciesPackage,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(4, 4, 3)?;
/// let domain = MicrophysicsDriverDomain::try_new(
///     shape,
///     1..3,
///     1..3,
///     0..3,
///     MicrophysicsBoundaryPolicy::open(),
/// )?;
/// let driver = MicrophysicsDriver::try_kessler(
///     domain,
///     MoistureSpeciesPackage::kessler(),
/// )?;
/// let mut workspace = driver.create_workspace(&backend)?;
///
/// let mut potential_temperature = backend.create_field(shape, 280.0_f32)?;
/// let mut moisture = vec![
///     backend.create_field(shape, 0.005_f32)?,
///     backend.create_field(shape, 0.001_f32)?,
///     backend.create_field(shape, 0.0005_f32)?,
/// ];
/// let density = backend.create_field(shape, 1.0_f32)?;
/// let exner = backend.create_field(shape, 0.95_f32)?;
/// let height = backend.create_field(shape, 100.0_f32)?;
/// let thickness = backend.create_field(shape, 150.0_f32)?;
/// let mut accumulated = backend.create_field(shape.horizontal_shape(), 0.0_f32)?;
/// let mut step = backend.create_field(shape.horizontal_shape(), 0.0_f32)?;
///
/// driver.apply(
///     &backend,
///     MicrophysicsDriverFields::new(
///         &mut potential_temperature,
///         &mut moisture,
///         &density,
///         &exner,
///         &height,
///         &thickness,
///         &mut accumulated,
///         &mut step,
///     ),
///     KesslerMicrophysicsParameters::try_from_wrf_defaults(30.0)?,
///     &[MicrophysicsTile::new(0..4, 0..4)],
///     &mut workspace,
/// )?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct MicrophysicsDriver {
    domain: MicrophysicsDriverDomain,
    dispatch: MicrophysicsDispatch,
}

impl MicrophysicsDriver {
    /// Creates a disabled driver that owns no scheme package or scratch.
    ///
    /// Applying this driver returns before reading or validating fields,
    /// matching the pinned `mp_physics == 0` preamble.
    pub const fn disabled(domain: MicrophysicsDriverDomain) -> Self {
        Self {
            domain,
            dispatch: MicrophysicsDispatch::Disabled,
        }
    }

    /// Binds Kessler to a domain and Registry moisture-species ordering.
    ///
    /// # Errors
    ///
    /// Returns an error if the selected scheme requires a moisture species
    /// the package does not carry, mirroring the pinned driver's fatal
    /// `arguments not present for calling kessler` rejection.
    pub fn try_kessler(
        domain: MicrophysicsDriverDomain,
        moisture_package: MoistureSpeciesPackage,
    ) -> MicrophysicsDriverResult<Self> {
        let species_selection = KesslerSpeciesSelection {
            water_vapor: moisture_package
                .require_index_of(MoistureSpecies::WaterVapor)?
                .as_usize(),
            cloud_water: moisture_package
                .require_index_of(MoistureSpecies::CloudWater)?
                .as_usize(),
            rain_water: moisture_package
                .require_index_of(MoistureSpecies::RainWater)?
                .as_usize(),
        };

        Ok(Self {
            domain,
            dispatch: MicrophysicsDispatch::Kessler(KesslerDispatch {
                moisture_package,
                species_selection,
            }),
        })
    }

    /// Returns the selected scheme.
    pub const fn scheme(&self) -> MicrophysicsScheme {
        match &self.dispatch {
            MicrophysicsDispatch::Disabled => MicrophysicsScheme::Disabled,
            MicrophysicsDispatch::Kessler(_) => MicrophysicsScheme::Kessler,
        }
    }

    /// Allocates reusable scheme scratch sized for the driver's domain.
    ///
    /// # Errors
    ///
    /// Returns an error if the domain does not satisfy the scheme's region
    /// requirements or the backend cannot allocate the workspace.
    pub fn create_workspace<Backend: KesslerMicrophysicsKernels>(
        &self,
        backend: &Backend,
    ) -> MicrophysicsDriverResult<MicrophysicsDriverWorkspace<Backend::Workspace>> {
        match &self.dispatch {
            MicrophysicsDispatch::Disabled => Ok(MicrophysicsDriverWorkspace::disabled()),
            MicrophysicsDispatch::Kessler(_) => {
                let region = self.domain_region()?;
                let workspace = backend.create_kessler_microphysics_workspace(&region)?;
                Ok(MicrophysicsDriverWorkspace::kessler(workspace))
            }
        }
    }

    /// Applies one microphysics update over every active tile.
    ///
    /// A disabled scheme returns without reading or mutating any field,
    /// mirroring the pinned driver's `mp_physics == 0` early return. All
    /// species, shapes, and tile regions are validated before any mutation;
    /// a validation error therefore leaves every field untouched.
    ///
    /// # Errors
    ///
    /// Returns an error if the moisture slice does not match the package,
    /// any field shape differs from the domain, any active tile forms an
    /// invalid kernel region, or kernel execution fails.
    pub fn apply<Backend: KesslerMicrophysicsKernels>(
        &self,
        backend: &Backend,
        fields: MicrophysicsDriverFields<'_, Backend::Field>,
        parameters: KesslerMicrophysicsParameters,
        tiles: &[MicrophysicsTile],
        workspace: &mut MicrophysicsDriverWorkspace<Backend::Workspace>,
    ) -> MicrophysicsDriverResult<()> {
        match &self.dispatch {
            MicrophysicsDispatch::Disabled => Ok(()),
            MicrophysicsDispatch::Kessler(kessler_dispatch) => {
                let workspace_scheme = workspace.scheme();
                let kessler_workspace = workspace.kessler_workspace_mut().ok_or(
                    MicrophysicsDriverError::WorkspaceSchemeMismatch {
                        driver_scheme: self.scheme(),
                        workspace_scheme,
                    },
                )?;
                self.apply_kessler(
                    backend,
                    fields,
                    parameters,
                    tiles,
                    kessler_workspace,
                    kessler_dispatch,
                )
            }
        }
    }

    fn apply_kessler<Backend: KesslerMicrophysicsKernels>(
        &self,
        backend: &Backend,
        fields: MicrophysicsDriverFields<'_, Backend::Field>,
        parameters: KesslerMicrophysicsParameters,
        tiles: &[MicrophysicsTile],
        workspace: &mut Backend::Workspace,
        dispatch: &KesslerDispatch,
    ) -> MicrophysicsDriverResult<()> {
        let MicrophysicsDriverFields {
            potential_temperature,
            moisture_species_fields,
            dry_air_density,
            exner_function,
            height,
            vertical_layer_thickness,
            accumulated_precipitation,
            step_precipitation,
        } = fields;

        self.validate_moisture_fields(&dispatch.moisture_package, moisture_species_fields)?;
        let field_shape = self.domain.field_shape();
        let precipitation_shape = field_shape.horizontal_shape();
        validate_field_shape(
            &*potential_temperature,
            KesslerMicrophysicsField::PotentialTemperature,
            field_shape,
        )?;
        validate_field_shape(
            dry_air_density,
            KesslerMicrophysicsField::DryAirDensity,
            field_shape,
        )?;
        validate_field_shape(
            exner_function,
            KesslerMicrophysicsField::ExnerFunction,
            field_shape,
        )?;
        validate_field_shape(height, KesslerMicrophysicsField::Height, field_shape)?;
        validate_field_shape(
            vertical_layer_thickness,
            KesslerMicrophysicsField::VerticalLayerThickness,
            field_shape,
        )?;
        validate_field_shape(
            &*accumulated_precipitation,
            KesslerMicrophysicsField::AccumulatedPrecipitation,
            precipitation_shape,
        )?;
        validate_field_shape(
            &*step_precipitation,
            KesslerMicrophysicsField::StepPrecipitation,
            precipitation_shape,
        )?;

        // Validate every active tile region before the first mutation so a
        // rejected later tile cannot leave earlier tiles partially updated.
        for tile in tiles {
            let _ = self.tile_region(tile)?;
        }

        for tile in tiles {
            let Some(region) = self.tile_region(tile)? else {
                continue;
            };
            let (water_vapor, cloud_water, rain_water) = select_kessler_species_mut(
                &mut *moisture_species_fields,
                dispatch.species_selection,
            )?;
            backend.apply_kessler_microphysics(
                KesslerMicrophysicsFields::new(
                    &mut *potential_temperature,
                    water_vapor,
                    cloud_water,
                    rain_water,
                    dry_air_density,
                    exner_function,
                    height,
                    vertical_layer_thickness,
                    &mut *accumulated_precipitation,
                    &mut *step_precipitation,
                ),
                parameters,
                &region,
                workspace,
            )?;
        }
        Ok(())
    }

    fn validate_moisture_fields<Field: FieldStorage<f32>>(
        &self,
        moisture_package: &MoistureSpeciesPackage,
        moisture_species_fields: &[Field],
    ) -> MicrophysicsDriverResult<()> {
        let expected = moisture_package.species_count();
        let actual = moisture_species_fields.len();
        if actual != expected {
            return Err(MicrophysicsDriverError::MoistureFieldCountMismatch { expected, actual });
        }
        let field_shape = self.domain.field_shape();
        for (species, field) in moisture_package
            .species()
            .iter()
            .zip(moisture_species_fields)
        {
            let actual_shape = field.shape();
            if actual_shape != field_shape {
                return Err(MicrophysicsDriverError::MoistureFieldShapeMismatch {
                    species: *species,
                    expected: field_shape,
                    actual: actual_shape,
                });
            }
        }
        Ok(())
    }

    fn domain_region(&self) -> MicrophysicsDriverResult<KesslerMicrophysicsRegion> {
        Ok(KesslerMicrophysicsRegion::try_new(
            self.domain.field_shape(),
            self.domain.west_east_range(),
            self.domain.south_north_range(),
            self.domain.bottom_top_range(),
        )?)
    }

    fn tile_region(
        &self,
        tile: &MicrophysicsTile,
    ) -> MicrophysicsDriverResult<Option<KesslerMicrophysicsRegion>> {
        let Some((west_east_range, south_north_range)) = self.domain.clip_tile(tile) else {
            return Ok(None);
        };
        let region = KesslerMicrophysicsRegion::try_new(
            self.domain.field_shape(),
            west_east_range,
            south_north_range,
            self.domain.bottom_top_range(),
        )?;
        Ok(Some(region))
    }
}

fn validate_field_shape<Field: FieldStorage<f32>>(
    field: &Field,
    field_name: KesslerMicrophysicsField,
    expected: GridShape,
) -> MicrophysicsDriverResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(MicrophysicsDriverError::Kernel(
            crate::KesslerMicrophysicsError::FieldShapeMismatch {
                field: field_name,
                expected,
                actual,
            },
        ));
    }
    Ok(())
}

fn select_kessler_species_mut<Field>(
    moisture_species_fields: &mut [Field],
    selection: KesslerSpeciesSelection,
) -> MicrophysicsDriverResult<(&mut Field, &mut Field, &mut Field)> {
    let mut water_vapor = None;
    let mut cloud_water = None;
    let mut rain_water = None;
    for (position, field) in moisture_species_fields.iter_mut().enumerate() {
        if position == selection.water_vapor {
            water_vapor = Some(field);
        } else if position == selection.cloud_water {
            cloud_water = Some(field);
        } else if position == selection.rain_water {
            rain_water = Some(field);
        }
    }
    match (water_vapor, cloud_water, rain_water) {
        (Some(vapor), Some(cloud), Some(rain)) => Ok((vapor, cloud, rain)),
        (None, _, _) => Err(MicrophysicsDriverError::MissingMoistureSpecies {
            species: MoistureSpecies::WaterVapor,
        }),
        (_, None, _) => Err(MicrophysicsDriverError::MissingMoistureSpecies {
            species: MoistureSpecies::CloudWater,
        }),
        (_, _, None) => Err(MicrophysicsDriverError::MissingMoistureSpecies {
            species: MoistureSpecies::RainWater,
        }),
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, CpuBackend, CpuField};

    use crate::{
        CpuKesslerMicrophysicsWorkspace, KesslerMicrophysicsError, MicrophysicsBoundaryPolicy,
        MicrophysicsDriverWorkspace,
    };

    use super::*;

    struct DriverFixture {
        potential_temperature: CpuField<f32>,
        moisture_species_fields: Vec<CpuField<f32>>,
        dry_air_density: CpuField<f32>,
        exner_function: CpuField<f32>,
        height: CpuField<f32>,
        vertical_layer_thickness: CpuField<f32>,
        accumulated_precipitation: CpuField<f32>,
        step_precipitation: CpuField<f32>,
    }

    fn fixture_shape() -> GridShape {
        GridShape::try_new(8, 7, 5).unwrap()
    }

    fn create_domain(policy: MicrophysicsBoundaryPolicy) -> MicrophysicsDriverDomain {
        MicrophysicsDriverDomain::try_new(fixture_shape(), 1..7, 1..6, 0..5, policy).unwrap()
    }

    fn create_driver(policy: MicrophysicsBoundaryPolicy) -> MicrophysicsDriver {
        MicrophysicsDriver::try_kessler(create_domain(policy), MoistureSpeciesPackage::kessler())
            .unwrap()
    }

    fn create_fixture(backend: &CpuBackend, package: &MoistureSpeciesPackage) -> DriverFixture {
        let shape = fixture_shape();
        let mut fixture = DriverFixture {
            potential_temperature: backend.create_field(shape, 0.0).unwrap(),
            moisture_species_fields: package
                .species()
                .iter()
                .map(|_| backend.create_field(shape, 0.0).unwrap())
                .collect(),
            dry_air_density: backend.create_field(shape, 0.0).unwrap(),
            exner_function: backend.create_field(shape, 0.0).unwrap(),
            height: backend.create_field(shape, 0.0).unwrap(),
            vertical_layer_thickness: backend.create_field(shape, 0.0).unwrap(),
            accumulated_precipitation: backend
                .create_field(shape.horizontal_shape(), 10.0)
                .unwrap(),
            step_precipitation: backend
                .create_field(shape.horizontal_shape(), -777.0)
                .unwrap(),
        };
        for south_north_index in 0..shape.south_north_points() {
            for bottom_top_index in 0..shape.bottom_top_points() {
                for west_east_index in 0..shape.west_east_points() {
                    let index =
                        linear_index(shape, west_east_index, bottom_top_index, south_north_index);
                    fixture.potential_temperature.values_mut()[index] =
                        278.0 + 0.7 * west_east_index as f32 + 0.3 * bottom_top_index as f32
                            - 0.4 * south_north_index as f32;
                    fixture.dry_air_density.values_mut()[index] =
                        1.15 - 0.08 * bottom_top_index as f32 + 0.01 * west_east_index as f32;
                    fixture.exner_function.values_mut()[index] =
                        0.99 - 0.015 * bottom_top_index as f32 + 0.002 * south_north_index as f32;
                    fixture.height.values_mut()[index] =
                        50.0 + 150.0 * bottom_top_index as f32 + 2.0 * west_east_index as f32;
                    fixture.vertical_layer_thickness.values_mut()[index] =
                        150.0 + 0.5 * west_east_index as f32;
                    for (species, field) in package
                        .species()
                        .iter()
                        .zip(&mut fixture.moisture_species_fields)
                    {
                        field.values_mut()[index] = species_value(
                            *species,
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                        );
                    }
                }
            }
        }
        fixture
    }

    fn species_value(
        species: MoistureSpecies,
        west_east_index: usize,
        bottom_top_index: usize,
        south_north_index: usize,
    ) -> f32 {
        match species {
            MoistureSpecies::WaterVapor => {
                0.002 + 0.001 * ((west_east_index + 2 * bottom_top_index) % 8) as f32
            }
            MoistureSpecies::CloudWater => {
                if (west_east_index + bottom_top_index) % 3 == 0 {
                    0.002
                } else {
                    0.0002
                }
            }
            MoistureSpecies::RainWater => {
                [0.0, 0.0005, 0.005, 0.02][(west_east_index + south_north_index) % 4]
            }
        }
    }

    fn apply_fixture(
        backend: &CpuBackend,
        driver: &MicrophysicsDriver,
        fixture: &mut DriverFixture,
        tiles: &[MicrophysicsTile],
    ) -> MicrophysicsDriverResult<()> {
        let mut workspace = driver.create_workspace(backend).unwrap();
        apply_fixture_with_workspace(backend, driver, fixture, tiles, &mut workspace)
    }

    fn apply_fixture_with_workspace(
        backend: &CpuBackend,
        driver: &MicrophysicsDriver,
        fixture: &mut DriverFixture,
        tiles: &[MicrophysicsTile],
        workspace: &mut MicrophysicsDriverWorkspace<CpuKesslerMicrophysicsWorkspace>,
    ) -> MicrophysicsDriverResult<()> {
        driver.apply(
            backend,
            MicrophysicsDriverFields::new(
                &mut fixture.potential_temperature,
                &mut fixture.moisture_species_fields,
                &fixture.dry_air_density,
                &fixture.exner_function,
                &fixture.height,
                &fixture.vertical_layer_thickness,
                &mut fixture.accumulated_precipitation,
                &mut fixture.step_precipitation,
            ),
            KesslerMicrophysicsParameters::try_from_wrf_defaults(60.0).unwrap(),
            tiles,
            workspace,
        )
    }

    fn mutable_state(fixture: &DriverFixture) -> Vec<Vec<u32>> {
        let mut state: Vec<Vec<u32>> = vec![field_bits(&fixture.potential_temperature)];
        for field in &fixture.moisture_species_fields {
            state.push(field_bits(field));
        }
        state.push(field_bits(&fixture.accumulated_precipitation));
        state.push(field_bits(&fixture.step_precipitation));
        state
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn linear_index(
        shape: GridShape,
        west_east_index: usize,
        bottom_top_index: usize,
        south_north_index: usize,
    ) -> usize {
        (south_north_index * shape.bottom_top_points() + bottom_top_index)
            * shape.west_east_points()
            + west_east_index
    }

    #[test]
    fn try_kessler_rejects_a_package_missing_a_required_species() {
        let package = MoistureSpeciesPackage::try_new(vec![
            MoistureSpecies::WaterVapor,
            MoistureSpecies::CloudWater,
        ])
        .unwrap();

        let result = MicrophysicsDriver::try_kessler(
            create_domain(MicrophysicsBoundaryPolicy::open()),
            package,
        );

        assert!(matches!(
            result,
            Err(MicrophysicsDriverError::MissingMoistureSpecies {
                species: MoistureSpecies::RainWater,
            })
        ));
    }

    #[test]
    fn disabled_scheme_ignores_missing_moisture_fields_and_mutates_nothing() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let driver =
            MicrophysicsDriver::disabled(create_domain(MicrophysicsBoundaryPolicy::open()));
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        fixture.moisture_species_fields.clear();
        let original = mutable_state(&fixture);

        apply_fixture(
            &backend,
            &driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
        )
        .unwrap();

        assert_eq!(mutable_state(&fixture), original);
    }

    #[test]
    fn disabled_scheme_workspace_has_no_kessler_region_requirements() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(2, 2, 1).unwrap();
        let domain = MicrophysicsDriverDomain::try_new(
            shape,
            0..2,
            0..2,
            0..1,
            MicrophysicsBoundaryPolicy::open(),
        )
        .unwrap();
        let driver = MicrophysicsDriver::disabled(domain);

        let workspace = driver.create_workspace(&backend).unwrap();

        assert_eq!(workspace.scheme(), MicrophysicsScheme::Disabled);
    }

    #[test]
    fn tiled_dispatch_matches_one_full_domain_tile_bitwise() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::new(true, false, 1));
        let mut split_fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        let mut whole_fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());

        apply_fixture(
            &backend,
            &driver,
            &mut split_fixture,
            &[
                MicrophysicsTile::new(0..8, 0..3),
                MicrophysicsTile::new(0..8, 3..7),
            ],
        )
        .unwrap();
        apply_fixture(
            &backend,
            &driver,
            &mut whole_fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
        )
        .unwrap();

        assert_eq!(mutable_state(&split_fixture), mutable_state(&whole_fixture));
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts_and_repeated_calls() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::new(true, false, 1));
        let tiles = [
            MicrophysicsTile::new(0..8, 0..4),
            MicrophysicsTile::new(0..8, 4..7),
        ];
        let mut single_fixture =
            create_fixture(&single_worker_backend, &MoistureSpeciesPackage::kessler());
        let mut four_fixture =
            create_fixture(&four_worker_backend, &MoistureSpeciesPackage::kessler());
        let mut single_workspace = driver.create_workspace(&single_worker_backend).unwrap();
        let mut four_workspace = driver.create_workspace(&four_worker_backend).unwrap();

        for _ in 0..2 {
            apply_fixture_with_workspace(
                &single_worker_backend,
                &driver,
                &mut single_fixture,
                &tiles,
                &mut single_workspace,
            )
            .unwrap();
            apply_fixture_with_workspace(
                &four_worker_backend,
                &driver,
                &mut four_fixture,
                &tiles,
                &mut four_workspace,
            )
            .unwrap();
        }

        assert_eq!(mutable_state(&single_fixture), mutable_state(&four_fixture));
    }

    #[test]
    fn reordered_package_produces_identical_species_results() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let canonical_package = MoistureSpeciesPackage::kessler();
        let reordered_package = MoistureSpeciesPackage::try_new(vec![
            MoistureSpecies::CloudWater,
            MoistureSpecies::RainWater,
            MoistureSpecies::WaterVapor,
        ])
        .unwrap();
        let domain = create_domain(MicrophysicsBoundaryPolicy::new(true, false, 1));
        let canonical_driver =
            MicrophysicsDriver::try_kessler(domain.clone(), canonical_package.clone()).unwrap();
        let reordered_driver =
            MicrophysicsDriver::try_kessler(domain, reordered_package.clone()).unwrap();
        let mut canonical_fixture = create_fixture(&backend, &canonical_package);
        let mut reordered_fixture = create_fixture(&backend, &reordered_package);
        let tiles = [MicrophysicsTile::new(0..8, 0..7)];

        apply_fixture(&backend, &canonical_driver, &mut canonical_fixture, &tiles).unwrap();
        apply_fixture(&backend, &reordered_driver, &mut reordered_fixture, &tiles).unwrap();

        for species in [
            MoistureSpecies::WaterVapor,
            MoistureSpecies::CloudWater,
            MoistureSpecies::RainWater,
        ] {
            let canonical_index = canonical_package.index_of(species).unwrap().as_usize();
            let reordered_index = reordered_package.index_of(species).unwrap().as_usize();
            assert_eq!(
                field_bits(&canonical_fixture.moisture_species_fields[canonical_index]),
                field_bits(&reordered_fixture.moisture_species_fields[reordered_index]),
            );
        }
    }

    #[test]
    fn inactive_tiles_leave_every_field_unchanged() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::new(true, false, 3));
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        let original = mutable_state(&fixture);

        apply_fixture(
            &backend,
            &driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..3)],
        )
        .unwrap();

        assert_eq!(mutable_state(&fixture), original);
    }

    #[test]
    fn moisture_count_mismatch_is_rejected_before_any_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::open());
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        fixture.moisture_species_fields.pop();
        let original = mutable_state(&fixture);

        let result = apply_fixture(
            &backend,
            &driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
        );

        assert_eq!(
            result,
            Err(MicrophysicsDriverError::MoistureFieldCountMismatch {
                expected: 3,
                actual: 2,
            })
        );
        assert_eq!(mutable_state(&fixture), original);
    }

    #[test]
    fn moisture_shape_mismatch_is_rejected_before_any_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::open());
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        let wrong_shape = GridShape::try_new(4, 4, 4).unwrap();
        fixture.moisture_species_fields[2] = backend.create_field(wrong_shape, 0.0).unwrap();
        let original = mutable_state(&fixture);

        let result = apply_fixture(
            &backend,
            &driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
        );

        assert_eq!(
            result,
            Err(MicrophysicsDriverError::MoistureFieldShapeMismatch {
                species: MoistureSpecies::RainWater,
                expected: fixture_shape(),
                actual: wrong_shape,
            })
        );
        assert_eq!(mutable_state(&fixture), original);
    }

    #[test]
    fn input_shape_mismatch_is_rejected_before_any_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::open());
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        let wrong_shape = GridShape::try_new(4, 4, 4).unwrap();
        fixture.exner_function = backend.create_field(wrong_shape, 1.0).unwrap();
        let original = mutable_state(&fixture);

        let result = apply_fixture(
            &backend,
            &driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
        );

        assert!(matches!(
            result,
            Err(MicrophysicsDriverError::Kernel(
                crate::KesslerMicrophysicsError::FieldShapeMismatch {
                    field: KesslerMicrophysicsField::ExnerFunction,
                    ..
                }
            ))
        ));
        assert_eq!(mutable_state(&fixture), original);
    }

    #[test]
    fn workspace_scheme_mismatch_is_rejected_before_any_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let kessler_driver = create_driver(MicrophysicsBoundaryPolicy::open());
        let disabled_driver =
            MicrophysicsDriver::disabled(create_domain(MicrophysicsBoundaryPolicy::open()));
        let mut workspace = disabled_driver.create_workspace(&backend).unwrap();
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        let original = mutable_state(&fixture);

        let result = apply_fixture_with_workspace(
            &backend,
            &kessler_driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
            &mut workspace,
        );

        assert_eq!(
            result,
            Err(MicrophysicsDriverError::WorkspaceSchemeMismatch {
                driver_scheme: MicrophysicsScheme::Kessler,
                workspace_scheme: MicrophysicsScheme::Disabled,
            })
        );
        assert_eq!(mutable_state(&fixture), original);
    }

    #[test]
    fn workspace_shape_mismatch_is_rejected_before_any_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let driver = create_driver(MicrophysicsBoundaryPolicy::open());
        let wrong_shape = GridShape::try_new(9, 8, 5).unwrap();
        let wrong_domain = MicrophysicsDriverDomain::try_new(
            wrong_shape,
            1..8,
            1..7,
            0..5,
            MicrophysicsBoundaryPolicy::open(),
        )
        .unwrap();
        let wrong_driver =
            MicrophysicsDriver::try_kessler(wrong_domain, MoistureSpeciesPackage::kessler())
                .unwrap();
        let mut workspace = wrong_driver.create_workspace(&backend).unwrap();
        let mut fixture = create_fixture(&backend, &MoistureSpeciesPackage::kessler());
        let original = mutable_state(&fixture);

        let result = apply_fixture_with_workspace(
            &backend,
            &driver,
            &mut fixture,
            &[MicrophysicsTile::new(0..8, 0..7)],
            &mut workspace,
        );

        assert_eq!(
            result,
            Err(MicrophysicsDriverError::Kernel(
                KesslerMicrophysicsError::WorkspaceShapeMismatch {
                    expected: fixture_shape(),
                    actual: wrong_shape,
                }
            ))
        );
        assert_eq!(mutable_state(&fixture), original);
    }
}
