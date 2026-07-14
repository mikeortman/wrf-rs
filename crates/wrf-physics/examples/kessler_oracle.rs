//! Emits complete raw-bit Kessler outputs for the pinned Fortran fixture.

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_physics::{
    KesslerMicrophysicsFields, KesslerMicrophysicsKernels, KesslerMicrophysicsParameters,
    KesslerMicrophysicsRegion,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = CpuBackend::try_with_worker_count(4)?;
    let shape = GridShape::try_new(6, 5, 5)?;
    let region = KesslerMicrophysicsRegion::try_new(shape, 1..5, 1..4, 0..5)?;
    let mut fields = create_fields(&backend, shape)?;
    let parameters = KesslerMicrophysicsParameters::try_from_wrf_defaults(60.0)?;
    let mut workspace = backend.create_kessler_microphysics_workspace(&region)?;

    backend.apply_kessler_microphysics(
        KesslerMicrophysicsFields::new(
            &mut fields.potential_temperature,
            &mut fields.water_vapor_mixing_ratio,
            &mut fields.cloud_water_mixing_ratio,
            &mut fields.rain_water_mixing_ratio,
            &fields.dry_air_density,
            &fields.exner_function,
            &fields.height,
            &fields.vertical_layer_thickness,
            &mut fields.accumulated_precipitation,
            &mut fields.step_precipitation,
        ),
        parameters,
        &region,
        &mut workspace,
    )?;

    print_field("potential_temperature", &fields.potential_temperature);
    print_field("water_vapor", &fields.water_vapor_mixing_ratio);
    print_field("cloud_water", &fields.cloud_water_mixing_ratio);
    print_field("rain_water", &fields.rain_water_mixing_ratio);
    print_field(
        "accumulated_precipitation",
        &fields.accumulated_precipitation,
    );
    print_field("step_precipitation", &fields.step_precipitation);
    Ok(())
}

struct KesslerOracleFields {
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
}

fn create_fields(
    backend: &CpuBackend,
    shape: GridShape,
) -> Result<KesslerOracleFields, Box<dyn std::error::Error>> {
    let mut fields = KesslerOracleFields {
        potential_temperature: backend.create_field(shape, 0.0)?,
        water_vapor_mixing_ratio: backend.create_field(shape, 0.0)?,
        cloud_water_mixing_ratio: backend.create_field(shape, 0.0)?,
        rain_water_mixing_ratio: backend.create_field(shape, 0.0)?,
        dry_air_density: backend.create_field(shape, 0.0)?,
        exner_function: backend.create_field(shape, 0.0)?,
        height: backend.create_field(shape, 0.0)?,
        vertical_layer_thickness: backend.create_field(shape, 0.0)?,
        accumulated_precipitation: backend.create_field(shape.horizontal_shape(), 0.0)?,
        step_precipitation: backend.create_field(shape.horizontal_shape(), 0.0)?,
    };

    for south_north_index in 0..shape.south_north_points() {
        for bottom_top_index in 0..shape.bottom_top_points() {
            for west_east_index in 0..shape.west_east_points() {
                let index =
                    linear_index(shape, west_east_index, bottom_top_index, south_north_index);
                fields.potential_temperature.values_mut()[index] =
                    278.0 + 0.7 * west_east_index as f32 + 0.3 * bottom_top_index as f32
                        - 0.4 * south_north_index as f32;
                fields.water_vapor_mixing_ratio.values_mut()[index] =
                    0.002 + 0.001 * ((west_east_index + 2 * bottom_top_index) % 8) as f32;
                fields.cloud_water_mixing_ratio.values_mut()[index] =
                    if (west_east_index + bottom_top_index) % 3 == 0 {
                        0.002
                    } else {
                        0.0002
                    };
                fields.rain_water_mixing_ratio.values_mut()[index] =
                    [0.0, 0.0005, 0.005, 0.02][(west_east_index + south_north_index) % 4];
                fields.dry_air_density.values_mut()[index] =
                    1.15 - 0.08 * bottom_top_index as f32 + 0.01 * west_east_index as f32;
                fields.exner_function.values_mut()[index] =
                    0.99 - 0.015 * bottom_top_index as f32 + 0.002 * south_north_index as f32;
                fields.height.values_mut()[index] =
                    50.0 + 150.0 * bottom_top_index as f32 + 2.0 * west_east_index as f32;
                fields.vertical_layer_thickness.values_mut()[index] =
                    150.0 + 0.5 * west_east_index as f32;
            }
        }
    }
    for south_north_index in 0..shape.south_north_points() {
        for west_east_index in 0..shape.west_east_points() {
            let index = south_north_index * shape.west_east_points() + west_east_index;
            fields.accumulated_precipitation.values_mut()[index] =
                10.0 + 0.25 * west_east_index as f32 + 0.5 * south_north_index as f32;
            fields.step_precipitation.values_mut()[index] = -777.0;
        }
    }
    Ok(fields)
}

fn print_field(label: &str, field: &CpuField<f32>) {
    for (index, value) in field.values().iter().copied().enumerate() {
        println!("{label} {index} {:08X}", value.to_bits());
    }
}

fn linear_index(
    shape: GridShape,
    west_east_index: usize,
    bottom_top_index: usize,
    south_north_index: usize,
) -> usize {
    (south_north_index * shape.bottom_top_points() + bottom_top_index) * shape.west_east_points()
        + west_east_index
}
