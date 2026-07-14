use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::generator_error::GeneratorResult;

pub(crate) struct CorpusWriter {
    writer: BufWriter<File>,
}

impl CorpusWriter {
    pub(crate) fn create(path: &Path) -> GeneratorResult<Self> {
        Ok(Self {
            writer: BufWriter::new(File::create(path)?),
        })
    }

    pub(crate) fn write_metadata(&mut self, values: &[i64]) -> GeneratorResult<()> {
        let mut separator = "";
        for value in values {
            write!(self.writer, "{separator}{value}")?;
            separator = " ";
        }
        writeln!(self.writer)?;
        Ok(())
    }

    pub(crate) fn write_bits(&mut self, bits: &[u32]) -> GeneratorResult<()> {
        for value in bits {
            writeln!(self.writer, "{}", *value as i32)?;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> GeneratorResult<()> {
        self.writer.flush()?;
        Ok(())
    }
}
