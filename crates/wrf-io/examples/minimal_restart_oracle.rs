//! Independent-parity driver for the Registry-selected WRF restart schema.

use std::error::Error;
use std::path::Path;

use wrf_io::{
    WrfAttributeValue, WrfDatasetView, WrfFileSchema, WrfGridDimensions, WrfNetcdfWriter,
    WrfRegistryRestartSchemaBuilder, WrfRestartComparer, WrfTimestamp, WrfVariableName,
    WrfVariableValues, WrfVariableView,
};
use wrf_registry::RegistryParser;

const REGISTRY: &str = r#"dimspec i 1 standard_domain x west_east
dimspec k 2 standard_domain z bottom_top
dimspec j 3 standard_domain y south_north
dimspec s - namelist=1:num_soil_layers z soil_layers
dimspec n - constant=2 z modes
dimspec c - constant=(0:5) c categories
dimspec a - constant=(2:8) c -
state real temperature ikj dyn_em 1 - ir "T " "potential temperature " "K "
state real u ikj dyn_em 1 X ir U "x-wind component" "m s-1"
state real v ikj dyn_em 1 Y ir V "y-wind component" "m s-1"
state real w ikj dyn_em 1 Z ir W "z-wind component" "m s-1"
state integer land_mask ji misc 1 - ir LANDMASK "land mask" "1"
state doubleprecision energy ikj dyn_em 1 - ir ENERGY "total energy" "J"
state logical active ij misc 1 - ir ACTIVE "active cell" "1"
state real soil s misc 1 - ir SOIL "soil state" "kg kg-1"
state real soil_staggered s misc 1 Z ir SOILSTAG "staggered soil state" "kg kg-1"
state real mode n misc 1 - ir MODE "mode state" "1"
state real mode_staggered n misc 1 Z ir MODESTAG "staggered mode state" "1"
state integer category c misc 1 - ir CATEGORY "category code" "1"
state real anonymous a misc 1 - ir ANON "anonymous coordinate" "1"
state real xtime - misc 1 - ir " ignored" "minutes since start" "minutes"
state real tendency ikj dyn_em 2 - ir TEND "time-level tendency" "K s-1"
state character*256 note - misc 1 - h NOTE "history only" "-"
"#;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    match arguments.as_slice() {
        [_, command, path] if command == "write" => write_fixture(path, 1),
        [_, command, path, repetitions] if command == "write-repeat" => {
            write_fixture(path, repetitions.parse()?)
        }
        [_, command, left, right] if command == "compare" => {
            WrfRestartComparer::compare_paths(left, right)?;
            Ok(())
        }
        [_, command] if command == "schema-summary" => write_schema_summary(),
        _ => Err(
            "usage: minimal_restart_oracle write PATH | write-repeat PATH COUNT | compare LEFT RIGHT | schema-summary"
                .into(),
        ),
    }
}

fn write_fixture(path: impl AsRef<Path>, repetitions: usize) -> Result<(), Box<dyn Error>> {
    let schema = fixture_schema()?;
    let times = *b"2000-09-18_16:42:01";
    let temperature = float_values(24, 100);
    let u = float_values(30, 200);
    let v = float_values(32, 300);
    let w = float_values(36, 400);
    let land_mask = int_values(12, 500)?;
    let energy = double_values(24, 600);
    let active = (0..12)
        .map(|index| i32::from(index % 3 == 0))
        .collect::<Vec<_>>();
    let soil = float_values(4, 700);
    let soil_staggered = float_values(4, 800);
    let mode = float_values(2, 900);
    let mode_staggered = float_values(2, 1_000);
    let category = int_values(6, 1_100)?;
    let anonymous = float_values(7, 1_200);
    let model_minutes = [60.0_f32];
    let tendency_1 = float_values(24, 1_300);
    let tendency_2 = float_values(24, 1_400);
    let dataset = WrfDatasetView::try_new(
        &schema,
        vec![
            view("Times", WrfVariableValues::Character(&times))?,
            view("T", WrfVariableValues::Float32(&temperature))?,
            view("U", WrfVariableValues::Float32(&u))?,
            view("V", WrfVariableValues::Float32(&v))?,
            view("W", WrfVariableValues::Float32(&w))?,
            view("LANDMASK", WrfVariableValues::Int32(&land_mask))?,
            view("ENERGY", WrfVariableValues::Float64(&energy))?,
            view("ACTIVE", WrfVariableValues::Int32(&active))?,
            view("SOIL", WrfVariableValues::Float32(&soil))?,
            view("SOILSTAG", WrfVariableValues::Float32(&soil_staggered))?,
            view("MODE", WrfVariableValues::Float32(&mode))?,
            view("MODESTAG", WrfVariableValues::Float32(&mode_staggered))?,
            view("CATEGORY", WrfVariableValues::Int32(&category))?,
            view("ANON", WrfVariableValues::Float32(&anonymous))?,
            view("XTIME", WrfVariableValues::Float32(&model_minutes))?,
            view("TEND_1", WrfVariableValues::Float32(&tendency_1))?,
            view("TEND_2", WrfVariableValues::Float32(&tendency_2))?,
        ],
    )?;

    for _ in 0..repetitions {
        WrfNetcdfWriter::write(&path, &dataset)?;
    }
    Ok(())
}

fn fixture_schema() -> Result<WrfFileSchema, Box<dyn Error>> {
    let registry = RegistryParser::parse("Registry.restart-oracle", REGISTRY)?;
    Ok(WrfRegistryRestartSchemaBuilder::new(
        &registry,
        WrfGridDimensions::try_new(4, 3, 2)?,
        WrfTimestamp::try_new("2000-09-18_16:42:01")?,
        WrfTimestamp::try_new("2000-09-18_16:00:00")?,
        12_000.0,
        12_000.0,
    )
    .with_namelist_value("num_soil_layers", 4)
    .try_build()?)
}

fn write_schema_summary() -> Result<(), Box<dyn Error>> {
    let schema = fixture_schema()?;
    for name in ["T", "LANDMASK", "XTIME"] {
        let variable_name = WrfVariableName::try_new(name)?;
        let variable = schema
            .variable(&variable_name)
            .ok_or_else(|| format!("fixture variable {name} is missing"))?;
        let dimensions = variable
            .dimensions()
            .iter()
            .filter(|dimension| dimension.as_str() != "Time")
            .map(|dimension| {
                let length = schema
                    .dimensions()
                    .iter()
                    .find(|candidate| candidate.name() == dimension)
                    .map(|candidate| candidate.length())
                    .ok_or_else(|| {
                        format!("fixture dimension {} is missing", dimension.as_str())
                    })?;
                Ok(format!("{}={length}", dimension.as_str()))
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?
            .join("|");
        let memory_order = variable
            .attributes()
            .iter()
            .find(|attribute| attribute.name() == "MemoryOrder")
            .and_then(|attribute| match attribute.value() {
                WrfAttributeValue::Text(value) => Some(value.replace(' ', "_")),
                _ => None,
            })
            .ok_or_else(|| format!("fixture variable {name} has no MemoryOrder"))?;
        println!("{name}|{dimensions}|MemoryOrder={memory_order}");
    }
    Ok(())
}

fn float_values(length: usize, offset: usize) -> Vec<f32> {
    let mut values = (0..length)
        .map(|index| (offset + index) as f32)
        .collect::<Vec<_>>();
    if length >= 3 {
        values[0] = f32::from_bits(0x8000_0000);
        values[1] = f32::from_bits(0x7fc0_1234);
        values[2] = f32::INFINITY;
    }
    values
}

fn double_values(length: usize, offset: usize) -> Vec<f64> {
    let mut values = (0..length)
        .map(|index| (offset + index) as f64)
        .collect::<Vec<_>>();
    if length >= 3 {
        values[0] = f64::from_bits(0x8000_0000_0000_0000);
        values[1] = f64::from_bits(0x7ff8_0000_0000_1234);
        values[2] = f64::INFINITY;
    }
    values
}

fn int_values(length: usize, offset: i32) -> Result<Vec<i32>, Box<dyn Error>> {
    (0..length)
        .map(|index| -> Result<i32, Box<dyn Error>> {
            let index = i32::try_from(index)?;
            offset
                .checked_add(index)
                .ok_or_else(|| "oracle integer value overflow".into())
        })
        .collect()
}

fn view<'a>(
    name: &str,
    values: WrfVariableValues<'a>,
) -> Result<WrfVariableView<'a>, Box<dyn Error>> {
    Ok(WrfVariableView::try_new(name, values)?)
}
