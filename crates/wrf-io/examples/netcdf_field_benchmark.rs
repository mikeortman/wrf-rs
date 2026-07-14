//! Bulk NetCDF-3 field-write benchmark used by the matched C comparison.

use std::error::Error;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use netcdf3::{DataSet, FileWriter, Version};

const X_LENGTH: usize = 256;
const Y_LENGTH: usize = 256;
const Z_LENGTH: usize = 64;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let [_, path, repetitions] = arguments.as_slice() else {
        return Err("usage: netcdf_field_benchmark PATH COUNT".into());
    };
    let repetitions: usize = repetitions.parse()?;
    let mut definition = DataSet::new();
    definition.set_unlimited_dim("Time", 1)?;
    definition.add_fixed_dim("bottom_top", Z_LENGTH)?;
    definition.add_fixed_dim("south_north", Y_LENGTH)?;
    definition.add_fixed_dim("west_east", X_LENGTH)?;
    definition.add_var_f32("THM", &["Time", "bottom_top", "south_north", "west_east"])?;
    let values: Vec<f32> = (0..X_LENGTH * Y_LENGTH * Z_LENGTH)
        .map(|index| index as f32 * 0.125)
        .collect();

    let start = Instant::now();
    for _ in 0..repetitions {
        write_field(path, &definition, &values)?;
    }
    println!("elapsed_seconds={:.6}", start.elapsed().as_secs_f64());
    Ok(())
}

fn write_field(
    path: impl AsRef<Path>,
    definition: &DataSet,
    values: &[f32],
) -> Result<(), Box<dyn Error>> {
    let output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    let buffered_output = BufWriter::with_capacity(1024 * 1024, output);
    let mut writer = FileWriter::open_seek_write(
        path.as_ref().to_string_lossy().as_ref(),
        Box::new(buffered_output),
    )
    .map_err(|error| format!("{error:?}"))?;
    writer
        .set_def(definition, Version::Offset64Bit, 0)
        .map_err(|error| format!("{error:?}"))?;
    writer
        .write_var_f32("THM", values)
        .map_err(|error| format!("{error:?}"))?;
    writer.close().map_err(|error| format!("{error:?}"))?;
    Ok(())
}
