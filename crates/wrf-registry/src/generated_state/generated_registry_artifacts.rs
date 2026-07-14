use std::fs;
use std::io;
use std::path::Path;

/// Selected code-generation outputs reproduced by the first Registry slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedRegistryArtifacts {
    pub(crate) state_struct: String,
    pub(crate) namelist_defines: String,
    pub(crate) namelist_defaults: String,
    pub(crate) namelist_statements: String,
    pub(crate) model_data_order: String,
    pub(crate) state_metadata: String,
}

impl GeneratedRegistryArtifacts {
    /// Returns `inc/state_struct.inc` exactly as WRF emits it for the slice.
    #[must_use]
    pub fn state_struct(&self) -> &str {
        &self.state_struct
    }

    /// Returns the dimension-aware namelist declarations.
    #[must_use]
    pub fn namelist_defines(&self) -> &str {
        &self.namelist_defines
    }

    /// Returns default assignments for runtime configuration entries.
    #[must_use]
    pub fn namelist_defaults(&self) -> &str {
        &self.namelist_defaults
    }

    /// Returns Fortran `NAMELIST` statements grouped by configured section.
    #[must_use]
    pub fn namelist_statements(&self) -> &str {
        &self.namelist_statements
    }

    /// Returns WRF's compile-time model data-order declaration.
    #[must_use]
    pub fn model_data_order(&self) -> &str {
        &self.model_data_order
    }

    /// Returns a normalized view of metadata represented in WRF allocation artifacts.
    #[must_use]
    pub fn state_metadata(&self) -> &str {
        &self.state_metadata
    }

    /// Writes the selected `.inc` files and metadata projection beneath `output_directory`.
    pub fn write_to(&self, output_directory: impl AsRef<Path>) -> io::Result<()> {
        let output_directory = output_directory.as_ref();
        fs::create_dir_all(output_directory)?;
        fs::write(
            output_directory.join("state_struct.inc"),
            &self.state_struct,
        )?;
        fs::write(
            output_directory.join("namelist_defines.inc"),
            &self.namelist_defines,
        )?;
        fs::write(
            output_directory.join("namelist_defaults.inc"),
            &self.namelist_defaults,
        )?;
        fs::write(
            output_directory.join("namelist_statements.inc"),
            &self.namelist_statements,
        )?;
        fs::write(
            output_directory.join("model_data_order.inc"),
            &self.model_data_order,
        )?;
        fs::write(
            output_directory.join("state_metadata.txt"),
            &self.state_metadata,
        )?;
        Ok(())
    }
}
