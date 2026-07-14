//! Emits complete raw-bit microphysics-driver outputs for the pinned
//! Fortran driver-branch fixture and checks worker-count determinism.

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_physics::{
    KesslerMicrophysicsParameters, MicrophysicsBoundaryPolicy, MicrophysicsDriver,
    MicrophysicsDriverDomain, MicrophysicsDriverFields, MicrophysicsScheme, MicrophysicsTile,
    MoistureSpecies, MoistureSpeciesPackage,
};

const QUIET_NAN_BITS: u32 = 0x7FC0_0000;
const POSITIVE_INFINITY_BITS: u32 = 0x7F80_0000;

struct OracleCase {
    name: &'static str,
    mp_physics: i32,
    specified: bool,
    channel_switch: bool,
    specified_zone_width: usize,
    call_count: usize,
    tiles: Vec<MicrophysicsTile>,
    package: MoistureSpeciesPackage,
    exceptional: bool,
}

struct OracleFields {
    potential_temperature: CpuField<f32>,
    moisture_species_fields: Vec<CpuField<f32>>,
    dry_air_density: CpuField<f32>,
    exner_function: CpuField<f32>,
    height: CpuField<f32>,
    vertical_layer_thickness: CpuField<f32>,
    accumulated_precipitation: CpuField<f32>,
    step_precipitation: CpuField<f32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for case in create_cases()? {
        let single_worker_lines = run_case(&case, 1)?;
        let four_worker_lines = run_case(&case, 4)?;
        if single_worker_lines != four_worker_lines {
            return Err(format!("case {} differs between one and four workers", case.name).into());
        }
        for line in four_worker_lines {
            println!("{line}");
        }
    }
    Ok(())
}

fn create_cases() -> Result<Vec<OracleCase>, Box<dyn std::error::Error>> {
    // These raw tiles reach the final allocated rows. Domain clipping removes
    // the staggered edges exactly like solve_em's I_END/J_END minima before
    // the driver applies its specified-zone policy.
    let full_tile = || MicrophysicsTile::new(1..8, 1..7);
    Ok(vec![
        OracleCase {
            name: "disabled",
            mp_physics: 0,
            specified: true,
            channel_switch: false,
            specified_zone_width: 1,
            call_count: 1,
            tiles: vec![full_tile()],
            package: MoistureSpeciesPackage::kessler(),
            exceptional: false,
        },
        OracleCase {
            name: "two_tile_specified",
            mp_physics: 1,
            specified: true,
            channel_switch: false,
            specified_zone_width: 1,
            call_count: 2,
            tiles: vec![
                MicrophysicsTile::new(1..8, 1..3),
                MicrophysicsTile::new(1..8, 3..7),
            ],
            package: MoistureSpeciesPackage::kessler(),
            exceptional: false,
        },
        OracleCase {
            name: "channel_switch",
            mp_physics: 1,
            specified: true,
            channel_switch: true,
            specified_zone_width: 1,
            call_count: 1,
            tiles: vec![full_tile()],
            package: MoistureSpeciesPackage::kessler(),
            exceptional: false,
        },
        OracleCase {
            name: "partial_and_inactive",
            mp_physics: 1,
            specified: true,
            channel_switch: false,
            specified_zone_width: 2,
            call_count: 1,
            tiles: vec![
                MicrophysicsTile::new(1..8, 1..2),
                MicrophysicsTile::new(1..5, 2..6),
            ],
            package: MoistureSpeciesPackage::kessler(),
            exceptional: false,
        },
        OracleCase {
            name: "open_boundaries",
            mp_physics: 1,
            specified: false,
            channel_switch: false,
            specified_zone_width: 1,
            call_count: 1,
            tiles: vec![full_tile()],
            package: MoistureSpeciesPackage::kessler(),
            exceptional: false,
        },
        OracleCase {
            name: "reordered_species",
            mp_physics: 1,
            specified: true,
            channel_switch: false,
            specified_zone_width: 1,
            call_count: 1,
            tiles: vec![full_tile()],
            package: MoistureSpeciesPackage::try_new(vec![
                MoistureSpecies::CloudWater,
                MoistureSpecies::WaterVapor,
                MoistureSpecies::RainWater,
            ])?,
            exceptional: false,
        },
        OracleCase {
            name: "exceptional",
            mp_physics: 1,
            specified: true,
            channel_switch: false,
            specified_zone_width: 1,
            call_count: 1,
            tiles: vec![full_tile()],
            package: MoistureSpeciesPackage::kessler(),
            exceptional: true,
        },
    ])
}

fn run_case(
    case: &OracleCase,
    worker_count: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let backend = CpuBackend::try_with_worker_count(worker_count)?;
    let shape = GridShape::try_new(8, 7, 5)?;
    let domain = MicrophysicsDriverDomain::try_new(
        shape,
        1..7,
        1..6,
        0..5,
        MicrophysicsBoundaryPolicy::new(
            case.specified,
            case.channel_switch,
            case.specified_zone_width,
        ),
    )?;
    let driver = match MicrophysicsScheme::try_from_mp_physics(case.mp_physics)? {
        MicrophysicsScheme::Disabled => MicrophysicsDriver::disabled(domain),
        MicrophysicsScheme::Kessler => {
            MicrophysicsDriver::try_kessler(domain, case.package.clone())?
        }
    };
    let mut workspace = driver.create_workspace(&backend)?;
    let mut fields = create_fields(&backend, shape, case)?;
    let parameters = KesslerMicrophysicsParameters::try_from_wrf_defaults(60.0)?;

    for _ in 0..case.call_count {
        driver.apply(
            &backend,
            MicrophysicsDriverFields::new(
                &mut fields.potential_temperature,
                &mut fields.moisture_species_fields,
                &fields.dry_air_density,
                &fields.exner_function,
                &fields.height,
                &fields.vertical_layer_thickness,
                &mut fields.accumulated_precipitation,
                &mut fields.step_precipitation,
            ),
            parameters,
            &case.tiles,
            &mut workspace,
        )?;
    }

    let mut lines = Vec::new();
    collect_field(
        &mut lines,
        case.name,
        "theta",
        &fields.potential_temperature,
    );
    for (position, field) in fields.moisture_species_fields.iter().enumerate() {
        let label = format!("moist_{}", position + 1);
        collect_field(&mut lines, case.name, &label, field);
    }
    collect_field(
        &mut lines,
        case.name,
        "rainnc",
        &fields.accumulated_precipitation,
    );
    collect_field(&mut lines, case.name, "rainncv", &fields.step_precipitation);
    Ok(lines)
}

fn create_fields(
    backend: &CpuBackend,
    shape: GridShape,
    case: &OracleCase,
) -> Result<OracleFields, Box<dyn std::error::Error>> {
    let mut fields = OracleFields {
        potential_temperature: backend.create_field(shape, 0.0)?,
        moisture_species_fields: case
            .package
            .species()
            .iter()
            .map(|_| backend.create_field(shape, 0.0))
            .collect::<Result<Vec<_>, _>>()?,
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
                fields.dry_air_density.values_mut()[index] =
                    1.15 - 0.08 * bottom_top_index as f32 + 0.01 * west_east_index as f32;
                fields.exner_function.values_mut()[index] =
                    0.99 - 0.015 * bottom_top_index as f32 + 0.002 * south_north_index as f32;
                fields.height.values_mut()[index] =
                    50.0 + 150.0 * bottom_top_index as f32 + 2.0 * west_east_index as f32;
                fields.vertical_layer_thickness.values_mut()[index] =
                    150.0 + 0.5 * west_east_index as f32;
                for (species, field) in case
                    .package
                    .species()
                    .iter()
                    .zip(&mut fields.moisture_species_fields)
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
    for south_north_index in 0..shape.south_north_points() {
        for west_east_index in 0..shape.west_east_points() {
            let index = south_north_index * shape.west_east_points() + west_east_index;
            fields.accumulated_precipitation.values_mut()[index] =
                10.0 + 0.25 * west_east_index as f32 + 0.5 * south_north_index as f32;
            fields.step_precipitation.values_mut()[index] = -777.0;
        }
    }
    if case.exceptional {
        seed_exceptional_values(&mut fields, shape, &case.package)?;
    }
    Ok(fields)
}

fn seed_exceptional_values(
    fields: &mut OracleFields,
    shape: GridShape,
    package: &MoistureSpeciesPackage,
) -> Result<(), Box<dyn std::error::Error>> {
    let quiet_nan = f32::from_bits(QUIET_NAN_BITS);
    let positive_infinity = f32::from_bits(POSITIVE_INFINITY_BITS);
    let species_position = |species: MoistureSpecies| -> Result<usize, String> {
        package
            .index_of(species)
            .map(|index| index.as_usize())
            .ok_or_else(|| format!("package does not carry {species}"))
    };

    fields.potential_temperature.values_mut()[linear_index(shape, 2, 1, 2)] = quiet_nan;
    let water_vapor = species_position(MoistureSpecies::WaterVapor)?;
    fields.moisture_species_fields[water_vapor].values_mut()[linear_index(shape, 3, 0, 3)] =
        positive_infinity;
    let rain_water = species_position(MoistureSpecies::RainWater)?;
    fields.moisture_species_fields[rain_water].values_mut()[linear_index(shape, 4, 2, 2)] =
        quiet_nan;
    let cloud_water = species_position(MoistureSpecies::CloudWater)?;
    fields.moisture_species_fields[cloud_water].values_mut()[linear_index(shape, 0, 0, 0)] =
        quiet_nan;
    Ok(())
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

fn collect_field(lines: &mut Vec<String>, case_name: &str, label: &str, field: &CpuField<f32>) {
    for (index, value) in field.values().iter().copied().enumerate() {
        lines.push(format!(
            "{case_name}.{label} {index} {:08X}",
            value.to_bits()
        ));
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
