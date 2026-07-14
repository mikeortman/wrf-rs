//! Independent parity driver for the minimal WRF restart schema.

use std::error::Error;
use std::path::Path;

use wrf_io::{
    WrfDatasetView, WrfFileKind, WrfFileSchema, WrfGridDimensions, WrfNetcdfWriter,
    WrfRestartComparer, WrfTimestamp, WrfVariableValues, WrfVariableView,
};

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
        _ => Err(
            "usage: minimal_restart_oracle write PATH | write-repeat PATH COUNT | compare LEFT RIGHT"
                .into(),
        ),
    }
}

fn write_fixture(path: impl AsRef<Path>, repetitions: usize) -> Result<(), Box<dyn Error>> {
    let schema = WrfFileSchema::try_minimal_arw(
        WrfFileKind::Restart,
        WrfGridDimensions::try_new(4, 3, 2)?,
        WrfTimestamp::try_new("2000-09-18_16:42:01")?,
        WrfTimestamp::try_new("2000-09-18_16:00:00")?,
        12_000.0,
        12_000.0,
    )?;
    let times = *b"2000-09-18_16:42:01";
    let u = values(30, 100);
    let v = values(32, 200);
    let w = values(36, 300);
    let ph = values(36, 400);
    let phb = values(36, 500);
    let temperature = values(24, 600);
    let mu = values(12, 700);
    let mub = values(12, 800);
    let pressure = values(24, 900);
    let base_pressure = values(24, 1_000);
    let water_vapor = values(24, 1_100);
    let model_minutes = [60.0_f32];
    let dataset = WrfDatasetView::try_new(
        &schema,
        vec![
            view("Times", WrfVariableValues::Character(&times))?,
            view("U", WrfVariableValues::Float32(&u))?,
            view("V", WrfVariableValues::Float32(&v))?,
            view("W", WrfVariableValues::Float32(&w))?,
            view("PH", WrfVariableValues::Float32(&ph))?,
            view("PHB", WrfVariableValues::Float32(&phb))?,
            view("THM", WrfVariableValues::Float32(&temperature))?,
            view("MU", WrfVariableValues::Float32(&mu))?,
            view("MUB", WrfVariableValues::Float32(&mub))?,
            view("P", WrfVariableValues::Float32(&pressure))?,
            view("PB", WrfVariableValues::Float32(&base_pressure))?,
            view("QVAPOR", WrfVariableValues::Float32(&water_vapor))?,
            view("XTIME", WrfVariableValues::Float32(&model_minutes))?,
        ],
    )?;
    for _ in 0..repetitions {
        WrfNetcdfWriter::write(&path, &dataset)?;
    }
    Ok(())
}

fn values(length: usize, offset: usize) -> Vec<f32> {
    (0..length).map(|index| (offset + index) as f32).collect()
}

fn view<'a>(
    name: &str,
    values: WrfVariableValues<'a>,
) -> Result<WrfVariableView<'a>, Box<dyn Error>> {
    Ok(WrfVariableView::try_new(name, values)?)
}
