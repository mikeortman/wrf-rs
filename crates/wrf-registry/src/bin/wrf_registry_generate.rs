//! Generates the selected WRF-compatible Registry artifacts from one source.
//!
//! Mirrors the WRF `registry` command line: `-DSYMBOL` flags define whole
//! symbol strings for `ifdef`/`ifndef` directives, and includes are resolved
//! against `./Registry/` and the source's own directory.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;

use wrf_registry::{RegistryArtifactGenerator, RegistryDefinitions, RegistryParser};

const USAGE: &str =
    "usage: wrf-registry-generate [-DSYMBOL]... <registry-source> <output-directory>";

fn main() -> Result<(), Box<dyn Error>> {
    let (definitions, [source_path, output_directory]) = parse_arguments(env::args_os().skip(1))?;

    let document = RegistryParser::parse_file(&source_path, &definitions)?;
    let artifacts = RegistryArtifactGenerator::generate(&document)?;
    artifacts.write_to(output_directory)?;
    Ok(())
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<(RegistryDefinitions, [PathBuf; 2]), &'static str> {
    let mut definitions = RegistryDefinitions::new();
    let mut positional_arguments = Vec::new();
    for argument in arguments {
        match argument.to_str().and_then(|value| value.strip_prefix("-D")) {
            Some(symbol) => definitions.define(symbol),
            None => positional_arguments.push(PathBuf::from(argument)),
        }
    }

    let paths = <[PathBuf; 2]>::try_from(positional_arguments).map_err(|_| USAGE)?;
    Ok((definitions, paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separates_definition_flags_from_paths() {
        let (definitions, paths) = parse_arguments([
            OsString::from("-DEM_CORE=1"),
            OsString::from("Registry/Registry.EM"),
            OsString::from("generated"),
        ])
        .unwrap();

        assert!(definitions.is_defined("EM_CORE=1"));
        assert_eq!(paths[0], PathBuf::from("Registry/Registry.EM"));
        assert_eq!(paths[1], PathBuf::from("generated"));
    }

    #[cfg(unix)]
    #[test]
    fn preserves_non_utf8_positional_paths() {
        use std::os::unix::ffi::OsStringExt;

        let source = OsString::from_vec(vec![b'R', 0xff]);
        let output = OsString::from_vec(vec![b'O', 0xfe]);
        let (_, paths) = parse_arguments([source.clone(), output.clone()]).unwrap();

        assert_eq!(paths[0], PathBuf::from(source));
        assert_eq!(paths[1], PathBuf::from(output));
    }
}
