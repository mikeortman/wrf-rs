//! Generates versioned, deterministic IEEE-754 input corpora shared by the
//! pinned WRF Fortran oracles and Rust unit tests.

mod column_mass_staggering_corpus;
mod corpus_generator;
mod corpus_writer;
mod deterministic_random;
mod generator_error;
mod held_suarez_corpus;
mod positive_definite_corpus;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use corpus_generator::ArwCorpusGenerator;
use generator_error::{GeneratorError, GeneratorResult};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ARW corpus generation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> GeneratorResult<()> {
    let mut arguments = env::args_os();
    let program_name = arguments
        .next()
        .unwrap_or_else(|| "wrf-arw-corpus-generator".into());
    let Some(output_directory) = arguments.next() else {
        return Err(GeneratorError::InvalidArguments {
            usage: format!(
                "usage: {} OUTPUT_DIRECTORY",
                PathBuf::from(program_name).display()
            ),
        });
    };
    if arguments.next().is_some() {
        return Err(GeneratorError::InvalidArguments {
            usage: format!(
                "usage: {} OUTPUT_DIRECTORY",
                PathBuf::from(program_name).display()
            ),
        });
    }

    ArwCorpusGenerator::new(PathBuf::from(output_directory)).generate()
}
