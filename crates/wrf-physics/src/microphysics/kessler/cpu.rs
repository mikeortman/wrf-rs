use wrf_compute::{CpuBackend, CpuField, FieldStorage};

use crate::{
    CpuKesslerMicrophysicsWorkspace, KesslerMicrophysicsError, KesslerMicrophysicsField,
    KesslerMicrophysicsFields, KesslerMicrophysicsKernels, KesslerMicrophysicsParameters,
    KesslerMicrophysicsRegion, KesslerMicrophysicsResult,
};

use super::algorithm::{
    KesslerSedimentation, KesslerSedimentationFields, KesslerWarmRainConversion,
    KesslerWarmRainFields,
};

impl KesslerMicrophysicsKernels for CpuBackend {
    type Field = CpuField<f32>;
    type Workspace = CpuKesslerMicrophysicsWorkspace;

    fn create_kessler_microphysics_workspace(
        &self,
        region: &KesslerMicrophysicsRegion,
    ) -> KesslerMicrophysicsResult<Self::Workspace> {
        CpuKesslerMicrophysicsWorkspace::try_new(self, region)
    }

    fn apply_kessler_microphysics(
        &self,
        fields: KesslerMicrophysicsFields<'_, Self::Field>,
        parameters: KesslerMicrophysicsParameters,
        region: &KesslerMicrophysicsRegion,
        workspace: &mut Self::Workspace,
    ) -> KesslerMicrophysicsResult<()> {
        validate_fields(&fields, region)?;
        if workspace.shape() != region.field_shape() {
            return Err(KesslerMicrophysicsError::WorkspaceShapeMismatch {
                expected: region.field_shape(),
                actual: workspace.shape(),
            });
        }

        let KesslerMicrophysicsFields {
            potential_temperature,
            water_vapor_mixing_ratio,
            cloud_water_mixing_ratio,
            rain_water_mixing_ratio,
            dry_air_density,
            exner_function,
            height,
            vertical_layer_thickness,
            accumulated_precipitation,
            step_precipitation,
        } = fields;

        let (production, column_scratch_by_worker) = workspace.execution_parts();
        KesslerSedimentation::apply(
            self,
            KesslerSedimentationFields {
                rain_water_mixing_ratio: rain_water_mixing_ratio.values(),
                dry_air_density: dry_air_density.values(),
                height: height.values(),
                vertical_layer_thickness: vertical_layer_thickness.values(),
                production: production.values_mut(),
                accumulated_precipitation: accumulated_precipitation.values_mut(),
                step_precipitation: step_precipitation.values_mut(),
            },
            parameters,
            region,
            column_scratch_by_worker,
        )?;

        KesslerWarmRainConversion::apply(
            self,
            KesslerWarmRainFields {
                potential_temperature: potential_temperature.values_mut(),
                water_vapor_mixing_ratio: water_vapor_mixing_ratio.values_mut(),
                cloud_water_mixing_ratio: cloud_water_mixing_ratio.values_mut(),
                rain_water_mixing_ratio: rain_water_mixing_ratio.values_mut(),
                dry_air_density: dry_air_density.values(),
                exner_function: exner_function.values(),
                production: workspace.production().values(),
            },
            parameters,
            region,
        )
    }
}

fn validate_fields(
    fields: &KesslerMicrophysicsFields<'_, CpuField<f32>>,
    region: &KesslerMicrophysicsRegion,
) -> KesslerMicrophysicsResult<()> {
    let field_shape = region.field_shape();
    validate_field_shape(
        fields.potential_temperature,
        KesslerMicrophysicsField::PotentialTemperature,
        field_shape,
    )?;
    validate_field_shape(
        fields.water_vapor_mixing_ratio,
        KesslerMicrophysicsField::WaterVaporMixingRatio,
        field_shape,
    )?;
    validate_field_shape(
        fields.cloud_water_mixing_ratio,
        KesslerMicrophysicsField::CloudWaterMixingRatio,
        field_shape,
    )?;
    validate_field_shape(
        fields.rain_water_mixing_ratio,
        KesslerMicrophysicsField::RainWaterMixingRatio,
        field_shape,
    )?;
    validate_field_shape(
        fields.dry_air_density,
        KesslerMicrophysicsField::DryAirDensity,
        field_shape,
    )?;
    validate_field_shape(
        fields.exner_function,
        KesslerMicrophysicsField::ExnerFunction,
        field_shape,
    )?;
    validate_field_shape(fields.height, KesslerMicrophysicsField::Height, field_shape)?;
    validate_field_shape(
        fields.vertical_layer_thickness,
        KesslerMicrophysicsField::VerticalLayerThickness,
        field_shape,
    )?;

    let precipitation_shape = region.precipitation_shape();
    validate_field_shape(
        fields.accumulated_precipitation,
        KesslerMicrophysicsField::AccumulatedPrecipitation,
        precipitation_shape,
    )?;
    validate_field_shape(
        fields.step_precipitation,
        KesslerMicrophysicsField::StepPrecipitation,
        precipitation_shape,
    )
}

fn validate_field_shape(
    field: &CpuField<f32>,
    field_name: KesslerMicrophysicsField,
    expected: wrf_compute::GridShape,
) -> KesslerMicrophysicsResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(KesslerMicrophysicsError::FieldShapeMismatch {
            field: field_name,
            expected,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let mut single_worker_fixture = create_fixture(&single_worker_backend);
        let mut four_worker_fixture = create_fixture(&four_worker_backend);

        apply_fixture(&single_worker_backend, &mut single_worker_fixture).unwrap();
        apply_fixture(&four_worker_backend, &mut four_worker_fixture).unwrap();

        assert_mutable_fields_equal(&single_worker_fixture, &four_worker_fixture);
    }

    #[test]
    fn rejects_shape_mismatch_before_mutating_fields() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = create_fixture(&backend);
        let original_temperature = fixture.potential_temperature.values().to_vec();
        fixture.dry_air_density = backend
            .create_field(GridShape::try_new(5, 5, 5).unwrap(), 1.0)
            .unwrap();

        let result = apply_fixture(&backend, &mut fixture);

        assert_eq!(
            result,
            Err(KesslerMicrophysicsError::FieldShapeMismatch {
                field: KesslerMicrophysicsField::DryAirDensity,
                expected: fixture.region.field_shape(),
                actual: GridShape::try_new(5, 5, 5).unwrap(),
            })
        );
        assert_eq!(fixture.potential_temperature.values(), original_temperature);
    }

    #[test]
    fn leaves_horizontal_halos_unchanged() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut fixture = create_fixture(&backend);
        let original_temperature = fixture.potential_temperature.values().to_vec();

        apply_fixture(&backend, &mut fixture).unwrap();

        let shape = fixture.region.field_shape();
        for south_north_index in 0..shape.south_north_points() {
            for bottom_top_index in 0..shape.bottom_top_points() {
                for west_east_index in 0..shape.west_east_points() {
                    if (1..5).contains(&west_east_index) && (1..4).contains(&south_north_index) {
                        continue;
                    }
                    let index =
                        linear_index(shape, west_east_index, bottom_top_index, south_north_index);
                    assert_eq!(
                        fixture.potential_temperature.values()[index].to_bits(),
                        original_temperature[index].to_bits()
                    );
                }
            }
        }
    }

    struct KesslerFixture {
        potential_temperature: CpuField<f32>,
        water_vapor_mixing_ratio: CpuField<f32>,
        cloud_water_mixing_ratio: CpuField<f32>,
        rain_water_mixing_ratio: CpuField<f32>,
        dry_air_density: CpuField<f32>,
        exner_function: CpuField<f32>,
        height: CpuField<f32>,
        vertical_layer_thickness: CpuField<f32>,
        accumulated_precipitation: CpuField<f32>,
        step_precipitation: CpuField<f32>,
        parameters: KesslerMicrophysicsParameters,
        region: KesslerMicrophysicsRegion,
        workspace: CpuKesslerMicrophysicsWorkspace,
    }

    fn create_fixture(backend: &CpuBackend) -> KesslerFixture {
        let shape = GridShape::try_new(6, 5, 5).unwrap();
        let region = KesslerMicrophysicsRegion::try_new(shape, 1..5, 1..4, 0..5).unwrap();
        let mut fixture = KesslerFixture {
            potential_temperature: backend.create_field(shape, -777.0).unwrap(),
            water_vapor_mixing_ratio: backend.create_field(shape, -777.0).unwrap(),
            cloud_water_mixing_ratio: backend.create_field(shape, -777.0).unwrap(),
            rain_water_mixing_ratio: backend.create_field(shape, -777.0).unwrap(),
            dry_air_density: backend.create_field(shape, 1.0).unwrap(),
            exner_function: backend.create_field(shape, 1.0).unwrap(),
            height: backend.create_field(shape, 0.0).unwrap(),
            vertical_layer_thickness: backend.create_field(shape, 150.0).unwrap(),
            accumulated_precipitation: backend
                .create_field(shape.horizontal_shape(), 10.0)
                .unwrap(),
            step_precipitation: backend
                .create_field(shape.horizontal_shape(), -777.0)
                .unwrap(),
            parameters: KesslerMicrophysicsParameters::try_from_wrf_defaults(60.0).unwrap(),
            workspace: backend
                .create_kessler_microphysics_workspace(&region)
                .unwrap(),
            region,
        };
        initialize_fixture_fields(&mut fixture);
        fixture
    }

    fn initialize_fixture_fields(fixture: &mut KesslerFixture) {
        let shape = fixture.region.field_shape();
        for south_north_index in 0..shape.south_north_points() {
            for bottom_top_index in 0..shape.bottom_top_points() {
                for west_east_index in 0..shape.west_east_points() {
                    let index =
                        linear_index(shape, west_east_index, bottom_top_index, south_north_index);
                    fixture.potential_temperature.values_mut()[index] =
                        278.0 + 0.7 * west_east_index as f32 + 0.3 * bottom_top_index as f32
                            - 0.4 * south_north_index as f32;
                    fixture.water_vapor_mixing_ratio.values_mut()[index] =
                        0.002 + 0.001 * ((west_east_index + 2 * bottom_top_index) % 8) as f32;
                    fixture.cloud_water_mixing_ratio.values_mut()[index] =
                        if (west_east_index + bottom_top_index) % 3 == 0 {
                            0.002
                        } else {
                            0.0002
                        };
                    fixture.rain_water_mixing_ratio.values_mut()[index] =
                        [0.0, 0.0005, 0.005, 0.02][(west_east_index + south_north_index) % 4];
                    fixture.dry_air_density.values_mut()[index] =
                        1.15 - 0.08 * bottom_top_index as f32 + 0.01 * west_east_index as f32;
                    fixture.exner_function.values_mut()[index] =
                        0.99 - 0.015 * bottom_top_index as f32 + 0.002 * south_north_index as f32;
                    fixture.height.values_mut()[index] =
                        50.0 + 150.0 * bottom_top_index as f32 + 2.0 * west_east_index as f32;
                    fixture.vertical_layer_thickness.values_mut()[index] =
                        150.0 + 0.5 * west_east_index as f32;
                }
            }
        }
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fixture: &mut KesslerFixture,
    ) -> KesslerMicrophysicsResult<()> {
        backend.apply_kessler_microphysics(
            KesslerMicrophysicsFields::new(
                &mut fixture.potential_temperature,
                &mut fixture.water_vapor_mixing_ratio,
                &mut fixture.cloud_water_mixing_ratio,
                &mut fixture.rain_water_mixing_ratio,
                &fixture.dry_air_density,
                &fixture.exner_function,
                &fixture.height,
                &fixture.vertical_layer_thickness,
                &mut fixture.accumulated_precipitation,
                &mut fixture.step_precipitation,
            ),
            fixture.parameters,
            &fixture.region,
            &mut fixture.workspace,
        )
    }

    fn assert_mutable_fields_equal(left: &KesslerFixture, right: &KesslerFixture) {
        assert_eq!(
            left.potential_temperature.values(),
            right.potential_temperature.values()
        );
        assert_eq!(
            left.water_vapor_mixing_ratio.values(),
            right.water_vapor_mixing_ratio.values()
        );
        assert_eq!(
            left.cloud_water_mixing_ratio.values(),
            right.cloud_water_mixing_ratio.values()
        );
        assert_eq!(
            left.rain_water_mixing_ratio.values(),
            right.rain_water_mixing_ratio.values()
        );
        assert_eq!(
            left.accumulated_precipitation.values(),
            right.accumulated_precipitation.values()
        );
        assert_eq!(
            left.step_precipitation.values(),
            right.step_precipitation.values()
        );
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
}
