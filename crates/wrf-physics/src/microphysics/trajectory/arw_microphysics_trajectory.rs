use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape};
use wrf_registry::ResolvedScalarArrayLayout;

use crate::{
    ArwMicrophysicsControls, ArwMicrophysicsError, ArwMicrophysicsField, ArwMicrophysicsResult,
    ArwMicrophysicsStage, ArwMicrophysicsStageView, ArwMicrophysicsState, ArwMicrophysicsWorkspace,
    KesslerMicrophysicsParameters, MicrophysicsDriver, MicrophysicsDriverDomain,
    MicrophysicsDriverError, MicrophysicsDriverFields, MicrophysicsTile, MoistureSpecies,
    MoistureSpeciesPackage,
};

const GRAVITY: f32 = 9.81;
const DRY_AIR_GAS_CONSTANT: f32 = 287.0;
const WATER_VAPOR_GAS_CONSTANT: f32 = 461.6;
const DRY_AIR_SPECIFIC_HEAT: f32 = 7.0 * DRY_AIR_GAS_CONSTANT / 2.0;
const REFERENCE_PRESSURE: f32 = 100_000.0;
const REFERENCE_POTENTIAL_TEMPERATURE: f32 = 300.0;
const EXNER_EXPONENT: f32 = DRY_AIR_GAS_CONSTANT / DRY_AIR_SPECIFIC_HEAT;
const WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO: f32 =
    WATER_VAPOR_GAS_CONSTANT / DRY_AIR_GAS_CONSTANT;

/// Complete Kessler-relevant ARW time-split microphysics trajectory.
///
/// The trajectory preserves the pinned `solve_em.F` order: prepare full
/// thermodynamic fields, invoke the Registry-selected driver once across all
/// tiles, then convert the scheme result back to perturbation state and
/// diabatic tendencies. Setup owns all numerical fields and scheme scratch;
/// warmed stepping may retain bounded, field-size-independent scheduler
/// bookkeeping allocations.
#[derive(Clone, Debug)]
pub struct ArwMicrophysicsTrajectory {
    domain: MicrophysicsDriverDomain,
    driver: MicrophysicsDriver,
    moisture_package: MoistureSpeciesPackage,
    water_vapor_index: usize,
    cloud_water_index: usize,
    controls: ArwMicrophysicsControls,
    kessler_parameters: KesslerMicrophysicsParameters,
}

impl ArwMicrophysicsTrajectory {
    /// Binds Kessler to a resolved Registry `moist` layout and ARW controls.
    ///
    /// # Errors
    ///
    /// Returns a typed error if Registry roles do not form a valid Kessler
    /// package or the timestep cannot configure the accepted Kessler kernel.
    pub fn try_kessler(
        domain: MicrophysicsDriverDomain,
        layout: &ResolvedScalarArrayLayout,
        controls: ArwMicrophysicsControls,
    ) -> ArwMicrophysicsResult<Self> {
        let moisture_package = MoistureSpeciesPackage::try_from_registry_layout(layout)?;
        let water_vapor_index = moisture_package
            .require_index_of(MoistureSpecies::WaterVapor)?
            .as_usize();
        let cloud_water_index = moisture_package
            .require_index_of(MoistureSpecies::CloudWater)?
            .as_usize();
        let driver = MicrophysicsDriver::try_kessler(domain.clone(), moisture_package.clone())?;
        let kessler_parameters =
            KesslerMicrophysicsParameters::try_from_wrf_defaults(controls.time_step())
                .map_err(MicrophysicsDriverError::from)?;
        Ok(Self {
            domain,
            driver,
            moisture_package,
            water_vapor_index,
            cloud_water_index,
            controls,
            kessler_parameters,
        })
    }

    /// Allocates every diagnostic, tendency, and scheme scratch field once.
    ///
    /// # Errors
    ///
    /// Returns a typed error if backend allocation fails.
    pub fn create_workspace(
        &self,
        backend: &CpuBackend,
    ) -> ArwMicrophysicsResult<ArwMicrophysicsWorkspace> {
        ArwMicrophysicsWorkspace::try_new(backend, &self.driver, self.domain.field_shape())
    }

    /// Applies one complete ARW time-split microphysics step.
    ///
    /// # Errors
    ///
    /// Returns before model-state or workspace mutation if the execution
    /// backend has a different worker count than the workspace, or if any state
    /// or workspace shape is invalid. Driver failures preserve the existing
    /// driver's atomicity contract; preparation writes only reusable scratch.
    pub fn apply_step(
        &self,
        backend: &CpuBackend,
        state: ArwMicrophysicsState<'_>,
        tiles: &[MicrophysicsTile],
        workspace: &mut ArwMicrophysicsWorkspace,
    ) -> ArwMicrophysicsResult<()> {
        self.apply_step_with_observer(backend, state, tiles, workspace, |_, _| {})
    }

    /// Applies one step and observes immutable stage intermediates in order.
    ///
    /// The observer is monomorphized and receives borrowed views, so enabling
    /// direct-oracle instrumentation adds neither allocation nor invalid public
    /// stage-transition operations.
    ///
    /// # Errors
    ///
    /// Returns the same typed failures as [`Self::apply_step`].
    pub fn apply_step_with_observer<Observer>(
        &self,
        backend: &CpuBackend,
        mut state: ArwMicrophysicsState<'_>,
        tiles: &[MicrophysicsTile],
        workspace: &mut ArwMicrophysicsWorkspace,
        mut observer: Observer,
    ) -> ArwMicrophysicsResult<()>
    where
        Observer: FnMut(ArwMicrophysicsStage, ArwMicrophysicsStageView<'_>),
    {
        self.validate_state(backend, &state, workspace)?;
        self.prepare(&state, tiles, workspace);
        observer(
            ArwMicrophysicsStage::Prepared,
            ArwMicrophysicsStageView::new(&state, workspace),
        );

        self.driver.apply(
            backend,
            MicrophysicsDriverFields::new(
                &mut workspace.full_potential_temperature,
                &mut *state.moisture_species_fields,
                &workspace.dry_air_density,
                &workspace.exner_function,
                &workspace.height,
                &workspace.vertical_layer_thickness,
                &mut *state.accumulated_precipitation,
                &mut *state.step_precipitation,
            ),
            self.kessler_parameters,
            tiles,
            &mut workspace.driver_workspace,
        )?;
        observer(
            ArwMicrophysicsStage::MicrophysicsApplied,
            ArwMicrophysicsStageView::new(&state, workspace),
        );

        self.finish(&mut state, tiles, workspace);
        observer(
            ArwMicrophysicsStage::Finished,
            ArwMicrophysicsStageView::new(&state, workspace),
        );
        Ok(())
    }

    fn validate_state(
        &self,
        backend: &CpuBackend,
        state: &ArwMicrophysicsState<'_>,
        workspace: &ArwMicrophysicsWorkspace,
    ) -> ArwMicrophysicsResult<()> {
        if workspace.worker_count != backend.worker_count() {
            return Err(ArwMicrophysicsError::WorkspaceWorkerCountMismatch {
                workspace_worker_count: workspace.worker_count,
                backend_worker_count: backend.worker_count(),
            });
        }
        let mass_shape = self.domain.field_shape();
        let w_shape = w_level_shape(mass_shape)?;
        let precipitation_shape = mass_shape.horizontal_shape();
        validate_shape(
            state.perturbation_potential_temperature,
            ArwMicrophysicsField::PerturbationPotentialTemperature,
            mass_shape,
        )?;
        validate_shape(
            state.perturbation_inverse_density,
            ArwMicrophysicsField::PerturbationInverseDensity,
            mass_shape,
        )?;
        validate_shape(
            state.base_inverse_density,
            ArwMicrophysicsField::BaseInverseDensity,
            mass_shape,
        )?;
        validate_shape(
            state.perturbation_pressure,
            ArwMicrophysicsField::PerturbationPressure,
            mass_shape,
        )?;
        validate_shape(
            state.base_pressure,
            ArwMicrophysicsField::BasePressure,
            mass_shape,
        )?;
        validate_shape(
            state.perturbation_geopotential,
            ArwMicrophysicsField::PerturbationGeopotential,
            w_shape,
        )?;
        validate_shape(
            state.base_geopotential,
            ArwMicrophysicsField::BaseGeopotential,
            w_shape,
        )?;
        validate_shape(
            state.accumulated_precipitation,
            ArwMicrophysicsField::AccumulatedPrecipitation,
            precipitation_shape,
        )?;
        validate_shape(
            state.step_precipitation,
            ArwMicrophysicsField::StepPrecipitation,
            precipitation_shape,
        )?;
        if workspace.mass_shape != mass_shape {
            return Err(ArwMicrophysicsError::FieldShapeMismatch {
                field: ArwMicrophysicsField::Workspace,
                expected: mass_shape,
                actual: workspace.mass_shape,
            });
        }
        let expected = self.moisture_package.species_count();
        let actual = state.moisture_species_fields.len();
        if actual != expected {
            return Err(
                MicrophysicsDriverError::MoistureFieldCountMismatch { expected, actual }.into(),
            );
        }
        for (species, field) in self
            .moisture_package
            .species()
            .iter()
            .zip(state.moisture_species_fields.iter())
        {
            if field.shape() != mass_shape {
                return Err(MicrophysicsDriverError::MoistureFieldShapeMismatch {
                    species: *species,
                    expected: mass_shape,
                    actual: field.shape(),
                }
                .into());
            }
        }
        Ok(())
    }

    fn prepare(
        &self,
        state: &ArwMicrophysicsState<'_>,
        tiles: &[MicrophysicsTile],
        workspace: &mut ArwMicrophysicsWorkspace,
    ) {
        let shape = self.domain.field_shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let w_bottom_top_points = bottom_top_points + 1;
        let bottom_top_range = self.domain.bottom_top_range();
        let water_vapor = state.moisture_species_fields[self.water_vapor_index].values();
        let cloud_water = state.moisture_species_fields[self.cloud_water_index].values();

        for tile in tiles {
            let Some((west_east_range, south_north_range)) = self.domain.clip_tile(tile) else {
                continue;
            };
            for south_north_index in south_north_range {
                for bottom_top_index in bottom_top_range.clone() {
                    for west_east_index in west_east_range.clone() {
                        let mass_index = linear_index(
                            west_east_points,
                            bottom_top_points,
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                        );
                        let lower_w_index = linear_index(
                            west_east_points,
                            w_bottom_top_points,
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                        );
                        let upper_w_index = linear_index(
                            west_east_points,
                            w_bottom_top_points,
                            west_east_index,
                            bottom_top_index + 1,
                            south_north_index,
                        );
                        let lower_height = (state.perturbation_geopotential.values()
                            [lower_w_index]
                            + state.base_geopotential.values()[lower_w_index])
                            / GRAVITY;
                        let upper_height = (state.perturbation_geopotential.values()
                            [upper_w_index]
                            + state.base_geopotential.values()[upper_w_index])
                            / GRAVITY;
                        let perturbation_theta =
                            state.perturbation_potential_temperature.values()[mass_index];
                        let full_theta = if self.controls.uses_moist_potential_temperature() {
                            (perturbation_theta + REFERENCE_POTENTIAL_TEMPERATURE)
                                / (1.0
                                    + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO
                                        * water_vapor[mass_index])
                        } else {
                            perturbation_theta + REFERENCE_POTENTIAL_TEMPERATURE
                        };
                        workspace.full_potential_temperature.values_mut()[mass_index] = full_theta;
                        workspace.h_diabatic.values_mut()[mass_index] = full_theta;
                        workspace.qv_diabatic.values_mut()[mass_index] = water_vapor[mass_index];
                        workspace.qc_diabatic.values_mut()[mass_index] = cloud_water[mass_index];
                        workspace.dry_air_density.values_mut()[mass_index] = 1.0
                            / (state.perturbation_inverse_density.values()[mass_index]
                                + state.base_inverse_density.values()[mass_index]);
                        workspace.exner_function.values_mut()[mass_index] =
                            ((state.perturbation_pressure.values()[mass_index]
                                + state.base_pressure.values()[mass_index])
                                / REFERENCE_PRESSURE)
                                .powf(EXNER_EXPONENT);
                        workspace.height.values_mut()[mass_index] =
                            0.5 * (lower_height + upper_height);
                        workspace.vertical_layer_thickness.values_mut()[mass_index] =
                            upper_height - lower_height;
                    }
                }
            }
        }
    }

    fn finish(
        &self,
        state: &mut ArwMicrophysicsState<'_>,
        tiles: &[MicrophysicsTile],
        workspace: &mut ArwMicrophysicsWorkspace,
    ) {
        let shape = self.domain.field_shape();
        let west_east_points = shape.west_east_points();
        let bottom_top_points = shape.bottom_top_points();
        let bottom_top_range = self.domain.bottom_top_range();
        let water_vapor = state.moisture_species_fields[self.water_vapor_index].values();
        let cloud_water = state.moisture_species_fields[self.cloud_water_index].values();
        let time_step = self.controls.time_step();
        let maximum_change = self.controls.maximum_potential_temperature_tendency() * time_step;

        for tile in tiles {
            let Some((west_east_range, south_north_range)) = self.domain.clip_tile(tile) else {
                continue;
            };
            for south_north_index in south_north_range {
                for bottom_top_index in bottom_top_range.clone() {
                    for west_east_index in west_east_range.clone() {
                        let index = linear_index(
                            west_east_points,
                            bottom_top_points,
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                        );
                        if !self.controls.is_heating_enabled() {
                            workspace.h_diabatic.values_mut()[index] = 0.0;
                            workspace.qv_diabatic.values_mut()[index] = 0.0;
                            workspace.qc_diabatic.values_mut()[index] = 0.0;
                            continue;
                        }

                        let theta_before = workspace.h_diabatic.values()[index];
                        let qv_before = workspace.qv_diabatic.values()[index];
                        let qc_before = workspace.qc_diabatic.values()[index];
                        let qv_tendency_change = water_vapor[index] - qv_before;
                        let qc_tendency_change = cloud_water[index] - qc_before;
                        let potential_temperature_change =
                            (workspace.full_potential_temperature.values()[index] - theta_before)
                                .min(maximum_change)
                                .max(-maximum_change);

                        if self.controls.uses_moist_potential_temperature() {
                            let updated_full_theta =
                                workspace.full_potential_temperature.values()[index];
                            let updated_qv = water_vapor[index];
                            let updated_perturbation_theta = theta_before
                                * (1.0 + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO * qv_before)
                                + potential_temperature_change
                                    * (1.0
                                        + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO * updated_qv)
                                + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO
                                    * qv_tendency_change
                                    * updated_full_theta
                                - REFERENCE_POTENTIAL_TEMPERATURE;
                            state.perturbation_potential_temperature.values_mut()[index] =
                                updated_perturbation_theta;
                            workspace
                                .perturbation_dry_potential_temperature
                                .values_mut()[index] = (updated_perturbation_theta
                                + REFERENCE_POTENTIAL_TEMPERATURE)
                                / (1.0 + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO * updated_qv)
                                - REFERENCE_POTENTIAL_TEMPERATURE;
                            workspace.h_diabatic.values_mut()[index] = (potential_temperature_change
                                * (1.0 + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO * updated_qv)
                                + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO
                                    * qv_tendency_change
                                    * updated_full_theta)
                                / time_step;
                        } else {
                            state.perturbation_potential_temperature.values_mut()[index] +=
                                potential_temperature_change;
                            workspace
                                .perturbation_dry_potential_temperature
                                .values_mut()[index] =
                                state.perturbation_potential_temperature.values()[index];
                            workspace.h_diabatic.values_mut()[index] =
                                potential_temperature_change / time_step;
                        }
                        workspace.qv_diabatic.values_mut()[index] = qv_tendency_change / time_step;
                        workspace.qc_diabatic.values_mut()[index] = qc_tendency_change / time_step;
                    }
                }
            }
        }
    }
}

fn validate_shape(
    field: &CpuField<f32>,
    field_name: ArwMicrophysicsField,
    expected: GridShape,
) -> ArwMicrophysicsResult<()> {
    let actual = field.shape();
    if actual == expected {
        return Ok(());
    }
    Err(ArwMicrophysicsError::FieldShapeMismatch {
        field: field_name,
        expected,
        actual,
    })
}

fn w_level_shape(mass_shape: GridShape) -> ArwMicrophysicsResult<GridShape> {
    let bottom_top_points = mass_shape
        .bottom_top_points()
        .checked_add(1)
        .ok_or(wrf_compute::ComputeError::GridPointCountOverflow)?;
    Ok(GridShape::try_new(
        mass_shape.west_east_points(),
        mass_shape.south_north_points(),
        bottom_top_points,
    )?)
}

const fn linear_index(
    west_east_points: usize,
    bottom_top_points: usize,
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
) -> usize {
    (south_north_index * bottom_top_points + bottom_top_index) * west_east_points + west_east_index
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, CpuBackend};
    use wrf_registry::{RegistryParser, RuntimeConfigurationChoice};

    use crate::MicrophysicsBoundaryPolicy;

    use super::*;

    #[derive(Clone)]
    struct Fixture {
        perturbation_potential_temperature: CpuField<f32>,
        moisture_species_fields: Vec<CpuField<f32>>,
        perturbation_inverse_density: CpuField<f32>,
        base_inverse_density: CpuField<f32>,
        perturbation_pressure: CpuField<f32>,
        base_pressure: CpuField<f32>,
        perturbation_geopotential: CpuField<f32>,
        base_geopotential: CpuField<f32>,
        accumulated_precipitation: CpuField<f32>,
        step_precipitation: CpuField<f32>,
    }

    impl Fixture {
        fn state(&mut self) -> ArwMicrophysicsState<'_> {
            ArwMicrophysicsState::new(
                &mut self.perturbation_potential_temperature,
                &mut self.moisture_species_fields,
                &self.perturbation_inverse_density,
                &self.base_inverse_density,
                &self.perturbation_pressure,
                &self.base_pressure,
                &self.perturbation_geopotential,
                &self.base_geopotential,
                &mut self.accumulated_precipitation,
                &mut self.step_precipitation,
            )
        }

        fn mutable_bits(&self) -> Vec<Vec<u32>> {
            let mut fields = vec![bits(&self.perturbation_potential_temperature)];
            fields.extend(self.moisture_species_fields.iter().map(bits));
            fields.push(bits(&self.accumulated_precipitation));
            fields.push(bits(&self.step_precipitation));
            fields
        }
    }

    fn shape() -> GridShape {
        GridShape::try_new(4, 3, 3).unwrap()
    }

    fn domain() -> MicrophysicsDriverDomain {
        MicrophysicsDriverDomain::try_new(
            shape(),
            0..4,
            0..3,
            0..3,
            MicrophysicsBoundaryPolicy::open(),
        )
        .unwrap()
    }

    fn resolved_layout(order: &str) -> ResolvedScalarArrayLayout {
        let source = format!(
            "dimspec i 1 standard_domain x west_east\n\
             dimspec k 2 standard_domain z bottom_top\n\
             dimspec j 3 standard_domain y south_north\n\
             rconfig integer mp_physics namelist,physics 1 -1 - mp_physics \"\" \"\"\n\
             state real - ikjftb moist 1 - - - - -\n\
             state real qv ikjftb moist 1 - - QVAPOR vapor 1\n\
             state real qc ikjftb moist 1 - - QCLOUD cloud 1\n\
             state real qr ikjftb moist 1 - - QRAIN rain 1\n\
             package kesslerscheme mp_physics==1 - moist:{order}\n"
        );
        let document = RegistryParser::parse("Registry.trajectory", &source).unwrap();
        document
            .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", 1)])
            .unwrap()
            .remove(0)
    }

    fn trajectory(order: &str) -> ArwMicrophysicsTrajectory {
        trajectory_with_controls(
            order,
            ArwMicrophysicsControls::try_from_wrf_defaults(60.0).unwrap(),
        )
    }

    fn trajectory_with_controls(
        order: &str,
        controls: ArwMicrophysicsControls,
    ) -> ArwMicrophysicsTrajectory {
        ArwMicrophysicsTrajectory::try_kessler(domain(), &resolved_layout(order), controls).unwrap()
    }

    fn fixture(backend: &CpuBackend, order: &[MoistureSpecies]) -> Fixture {
        let mass_shape = shape();
        let w_shape = GridShape::try_new(4, 3, 4).unwrap();
        let mut base_geopotential = backend.create_field(w_shape, 0.0).unwrap();
        for south_north_index in 0..w_shape.south_north_points() {
            for bottom_top_index in 0..w_shape.bottom_top_points() {
                for west_east_index in 0..w_shape.west_east_points() {
                    let index = linear_index(
                        w_shape.west_east_points(),
                        w_shape.bottom_top_points(),
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                    );
                    base_geopotential.values_mut()[index] = GRAVITY
                        * (100.0 + 150.0 * bottom_top_index as f32 + west_east_index as f32);
                }
            }
        }
        let moisture_species_fields = order
            .iter()
            .map(|species| {
                backend
                    .create_field(
                        mass_shape,
                        match species {
                            MoistureSpecies::WaterVapor => 0.008,
                            MoistureSpecies::CloudWater => 0.001,
                            MoistureSpecies::RainWater => 0.002,
                        },
                    )
                    .unwrap()
            })
            .collect();
        Fixture {
            perturbation_potential_temperature: backend.create_field(mass_shape, 5.0).unwrap(),
            moisture_species_fields,
            perturbation_inverse_density: backend.create_field(mass_shape, 0.2).unwrap(),
            base_inverse_density: backend.create_field(mass_shape, 0.8).unwrap(),
            perturbation_pressure: backend.create_field(mass_shape, 0.0).unwrap(),
            base_pressure: backend
                .create_field(mass_shape, REFERENCE_PRESSURE)
                .unwrap(),
            perturbation_geopotential: backend.create_field(w_shape, 0.0).unwrap(),
            base_geopotential,
            accumulated_precipitation: backend
                .create_field(mass_shape.horizontal_shape(), 10.0)
                .unwrap(),
            step_precipitation: backend
                .create_field(mass_shape.horizontal_shape(), -1.0)
                .unwrap(),
        }
    }

    fn canonical_species() -> [MoistureSpecies; 3] {
        [
            MoistureSpecies::WaterVapor,
            MoistureSpecies::CloudWater,
            MoistureSpecies::RainWater,
        ]
    }

    fn bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn workspace_bits(workspace: &ArwMicrophysicsWorkspace) -> Vec<Vec<u32>> {
        vec![
            bits(&workspace.full_potential_temperature),
            bits(&workspace.dry_air_density),
            bits(&workspace.exner_function),
            bits(&workspace.height),
            bits(&workspace.vertical_layer_thickness),
            bits(&workspace.h_diabatic),
            bits(&workspace.qv_diabatic),
            bits(&workspace.qc_diabatic),
            bits(&workspace.perturbation_dry_potential_temperature),
        ]
    }

    #[test]
    fn observes_pinned_preparation_driver_and_finish_order() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let trajectory = trajectory("qv,qc,qr");
        let mut fixture = fixture(&backend, &canonical_species());
        let mut workspace = trajectory.create_workspace(&backend).unwrap();
        let mut observed_stages = Vec::new();
        let mut prepared_values = None;

        trajectory
            .apply_step_with_observer(
                &backend,
                fixture.state(),
                &[MicrophysicsTile::new(0..4, 0..3)],
                &mut workspace,
                |stage, view| {
                    observed_stages.push(stage);
                    if stage == ArwMicrophysicsStage::Prepared {
                        prepared_values = Some((
                            view.full_potential_temperature().values()[0],
                            view.dry_air_density().values()[0],
                            view.exner_function().values()[0],
                            view.height().values()[0],
                            view.vertical_layer_thickness().values()[0],
                            view.h_diabatic().values()[0],
                            view.qv_diabatic().values()[0],
                            view.qc_diabatic().values()[0],
                        ));
                    }
                },
            )
            .unwrap();

        assert_eq!(
            observed_stages,
            [
                ArwMicrophysicsStage::Prepared,
                ArwMicrophysicsStage::MicrophysicsApplied,
                ArwMicrophysicsStage::Finished,
            ]
        );
        let expected_theta = (5.0 + REFERENCE_POTENTIAL_TEMPERATURE)
            / (1.0 + WATER_VAPOR_TO_DRY_AIR_GAS_CONSTANT_RATIO * 0.008);
        let lower_height = (GRAVITY * 100.0) / GRAVITY;
        let upper_height = (GRAVITY * 250.0) / GRAVITY;
        assert_eq!(
            prepared_values,
            Some((
                expected_theta,
                1.0,
                1.0,
                0.5 * (lower_height + upper_height),
                upper_height - lower_height,
                expected_theta,
                0.008,
                0.001,
            ))
        );
    }

    #[test]
    fn rejects_wrong_w_level_shape_before_model_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let trajectory = trajectory("qv,qc,qr");
        let mut fixture = fixture(&backend, &canonical_species());
        fixture.perturbation_geopotential = backend.create_field(shape(), 0.0).unwrap();
        let original = fixture.mutable_bits();
        let mut workspace = trajectory.create_workspace(&backend).unwrap();

        let result = trajectory.apply_step(
            &backend,
            fixture.state(),
            &[MicrophysicsTile::new(0..4, 0..3)],
            &mut workspace,
        );

        assert!(matches!(
            result,
            Err(ArwMicrophysicsError::FieldShapeMismatch {
                field: ArwMicrophysicsField::PerturbationGeopotential,
                ..
            })
        ));
        assert_eq!(fixture.mutable_bits(), original);
    }

    #[test]
    fn rejects_workspace_worker_mismatch_before_model_or_workspace_mutation() {
        let one_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let trajectory = trajectory("qv,qc,qr");
        let mut fixture = fixture(&one_worker_backend, &canonical_species());
        let original_model = fixture.mutable_bits();
        let mut workspace = trajectory.create_workspace(&one_worker_backend).unwrap();
        let original_workspace = workspace_bits(&workspace);

        let result = trajectory.apply_step(
            &four_worker_backend,
            fixture.state(),
            &[MicrophysicsTile::new(0..4, 0..3)],
            &mut workspace,
        );

        assert_eq!(
            result,
            Err(ArwMicrophysicsError::WorkspaceWorkerCountMismatch {
                workspace_worker_count: 1,
                backend_worker_count: 4,
            })
        );
        assert_eq!(fixture.mutable_bits(), original_model);
        assert_eq!(workspace_bits(&workspace), original_workspace);
    }

    #[test]
    fn late_moisture_validation_failure_preserves_model_and_workspace_bits() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let trajectory = trajectory("qv,qc,qr");
        let mut fixture = fixture(&backend, &canonical_species());
        fixture.moisture_species_fields[2] = backend
            .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
            .unwrap();
        let original_model = fixture.mutable_bits();
        let mut workspace = trajectory.create_workspace(&backend).unwrap();
        workspace.full_potential_temperature.values_mut().fill(-9.0);
        workspace.h_diabatic.values_mut().fill(-8.0);
        let original_workspace = workspace_bits(&workspace);

        let result = trajectory.apply_step(
            &backend,
            fixture.state(),
            &[MicrophysicsTile::new(0..4, 0..3)],
            &mut workspace,
        );

        assert!(matches!(
            result,
            Err(ArwMicrophysicsError::Driver(
                MicrophysicsDriverError::MoistureFieldShapeMismatch {
                    species: MoistureSpecies::RainWater,
                    ..
                }
            ))
        ));
        assert_eq!(fixture.mutable_bits(), original_model);
        assert_eq!(workspace_bits(&workspace), original_workspace);
    }

    #[test]
    fn dry_theta_zero_limit_preserves_theta_and_reports_zero_heating() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let controls = ArwMicrophysicsControls::try_new(60.0, false, true, 0.0).unwrap();
        let trajectory = trajectory_with_controls("qv,qc,qr", controls);
        let mut fixture = fixture(&backend, &canonical_species());
        let original_theta = bits(&fixture.perturbation_potential_temperature);
        let mut workspace = trajectory.create_workspace(&backend).unwrap();

        trajectory
            .apply_step(
                &backend,
                fixture.state(),
                &[MicrophysicsTile::new(0..4, 0..3)],
                &mut workspace,
            )
            .unwrap();

        assert_eq!(
            bits(&fixture.perturbation_potential_temperature),
            original_theta
        );
        assert!(
            workspace
                .latent_heating_tendency()
                .values()
                .iter()
                .all(|value| *value == 0.0)
        );
        assert_eq!(
            bits(workspace.perturbation_dry_potential_temperature()),
            original_theta
        );
    }

    #[test]
    fn disabled_heating_preserves_theta_and_zeros_all_diabatic_tendencies() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let controls = ArwMicrophysicsControls::try_new(60.0, true, false, 10.0).unwrap();
        let trajectory = trajectory_with_controls("qv,qc,qr", controls);
        let mut fixture = fixture(&backend, &canonical_species());
        let original_theta = bits(&fixture.perturbation_potential_temperature);
        let mut workspace = trajectory.create_workspace(&backend).unwrap();

        trajectory
            .apply_step(
                &backend,
                fixture.state(),
                &[MicrophysicsTile::new(0..4, 0..3)],
                &mut workspace,
            )
            .unwrap();

        assert_eq!(
            bits(&fixture.perturbation_potential_temperature),
            original_theta
        );
        for tendency in [
            workspace.latent_heating_tendency(),
            workspace.water_vapor_tendency(),
            workspace.cloud_water_tendency(),
        ] {
            assert!(
                tendency
                    .values()
                    .iter()
                    .all(|value| value.to_bits() == 0.0_f32.to_bits())
            );
        }
    }

    #[test]
    fn inactive_tile_preserves_model_and_workspace_bits() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let trajectory = trajectory("qv,qc,qr");
        let mut fixture = fixture(&backend, &canonical_species());
        let original_model = fixture.mutable_bits();
        let mut workspace = trajectory.create_workspace(&backend).unwrap();
        let original_workspace = workspace_bits(&workspace);

        trajectory
            .apply_step(
                &backend,
                fixture.state(),
                &[MicrophysicsTile::new(8..10, 8..10)],
                &mut workspace,
            )
            .unwrap();

        assert_eq!(fixture.mutable_bits(), original_model);
        assert_eq!(workspace_bits(&workspace), original_workspace);
    }

    #[test]
    fn recreated_workspace_checkpoint_matches_uninterrupted_three_step_trajectory() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let trajectory = trajectory("qv,qc,qr");
        let mut uninterrupted = fixture(&backend, &canonical_species());
        let mut restarted = uninterrupted.clone();
        let mut uninterrupted_workspace = trajectory.create_workspace(&backend).unwrap();
        let mut restarted_workspace = trajectory.create_workspace(&backend).unwrap();
        let tiles = [MicrophysicsTile::new(0..4, 0..3)];

        for _ in 0..3 {
            trajectory
                .apply_step(
                    &backend,
                    uninterrupted.state(),
                    &tiles,
                    &mut uninterrupted_workspace,
                )
                .unwrap();
        }
        trajectory
            .apply_step(
                &backend,
                restarted.state(),
                &tiles,
                &mut restarted_workspace,
            )
            .unwrap();
        restarted_workspace = trajectory.create_workspace(&backend).unwrap();
        for _ in 0..2 {
            trajectory
                .apply_step(
                    &backend,
                    restarted.state(),
                    &tiles,
                    &mut restarted_workspace,
                )
                .unwrap();
        }

        assert_eq!(uninterrupted.mutable_bits(), restarted.mutable_bits());
        assert_eq!(
            workspace_bits(&uninterrupted_workspace),
            workspace_bits(&restarted_workspace)
        );
    }

    #[test]
    fn reordered_registry_species_are_bitwise_deterministic_across_workers() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let order = [
            MoistureSpecies::RainWater,
            MoistureSpecies::WaterVapor,
            MoistureSpecies::CloudWater,
        ];
        let trajectory = trajectory("qr,qv,qc");
        let mut one_fixture = fixture(&one_worker, &order);
        let mut four_fixture = fixture(&four_workers, &order);
        let mut one_workspace = trajectory.create_workspace(&one_worker).unwrap();
        let mut four_workspace = trajectory.create_workspace(&four_workers).unwrap();
        let tiles = [MicrophysicsTile::new(0..4, 0..3)];

        trajectory
            .apply_step(&one_worker, one_fixture.state(), &tiles, &mut one_workspace)
            .unwrap();
        trajectory
            .apply_step(
                &four_workers,
                four_fixture.state(),
                &tiles,
                &mut four_workspace,
            )
            .unwrap();

        assert_eq!(one_fixture.mutable_bits(), four_fixture.mutable_bits());
        assert_eq!(
            bits(one_workspace.latent_heating_tendency()),
            bits(four_workspace.latent_heating_tendency())
        );
        assert_eq!(
            bits(one_workspace.water_vapor_tendency()),
            bits(four_workspace.water_vapor_tendency())
        );
        assert_eq!(
            bits(one_workspace.cloud_water_tendency()),
            bits(four_workspace.cloud_water_tendency())
        );
    }
}
