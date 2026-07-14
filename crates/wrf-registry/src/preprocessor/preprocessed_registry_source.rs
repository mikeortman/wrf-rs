use std::sync::Arc;

use crate::SourceLocation;
use crate::parser::logical_line::LogicalLine;

/// Include-expanded, conditional-filtered logical Registry lines.
///
/// Lines appear in expansion order: an `include` directive is replaced by the
/// included file's lines at that point, exactly as WRF's `pre_parse` splices
/// files into its preprocessed stream. Each line keeps the physical file and
/// first line number it was written on, so downstream parse diagnostics point
/// at the original source even across nested includes.
#[derive(Debug)]
pub struct PreprocessedRegistrySource {
    pub(crate) root_name: Arc<str>,
    pub(crate) lines: Vec<LogicalLine>,
}

impl PreprocessedRegistrySource {
    /// Returns the name of the root source handed to the preprocessor.
    #[must_use]
    pub fn root_name(&self) -> &str {
        &self.root_name
    }

    /// Iterates over the surviving logical lines and their physical locations.
    pub fn lines(&self) -> impl Iterator<Item = (&str, &SourceLocation)> {
        self.lines
            .iter()
            .map(|line| (line.text.as_str(), &line.location))
    }
}
