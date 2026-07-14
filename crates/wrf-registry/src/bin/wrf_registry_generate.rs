//! Generates the selected WRF-compatible Registry artifacts from one source.

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use wrf_registry::{RegistryArtifactGenerator, RegistryParser};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let source_path = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("usage: wrf-registry-generate <registry-source> <output-directory>")?;
    let output_directory = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("usage: wrf-registry-generate <registry-source> <output-directory>")?;
    if arguments.next().is_some() {
        return Err("usage: wrf-registry-generate <registry-source> <output-directory>".into());
    }

    let source = fs::read_to_string(&source_path)?;
    let document = RegistryParser::parse(source_path.to_string_lossy().as_ref(), &source)?;
    let artifacts = RegistryArtifactGenerator::generate(&document)?;
    artifacts.write_to(output_directory)?;
    Ok(())
}
