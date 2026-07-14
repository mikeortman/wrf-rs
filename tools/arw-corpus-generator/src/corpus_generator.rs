use std::fs;
use std::path::PathBuf;

use crate::column_mass_staggering_corpus::ColumnMassStaggeringCorpus;
use crate::generator_error::GeneratorResult;
use crate::held_suarez_corpus::HeldSuarezCorpus;
use crate::positive_definite_corpus::PositiveDefiniteCorpus;

pub(crate) struct ArwCorpusGenerator {
    output_directory: PathBuf,
}

impl ArwCorpusGenerator {
    pub(crate) const fn new(output_directory: PathBuf) -> Self {
        Self { output_directory }
    }

    pub(crate) fn generate(&self) -> GeneratorResult<()> {
        fs::create_dir_all(&self.output_directory)?;
        PositiveDefiniteCorpus::write(&self.output_directory)?;
        HeldSuarezCorpus::write(&self.output_directory)?;
        ColumnMassStaggeringCorpus::write(&self.output_directory)?;
        Ok(())
    }
}
