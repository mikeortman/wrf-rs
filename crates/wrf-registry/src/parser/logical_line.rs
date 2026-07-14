use std::sync::Arc;

use crate::{RegistryParseError, RegistryParseErrorKind, RegistryResult, SourceLocation};

/// One backslash-joined Registry line located at its first physical line.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LogicalLine {
    pub(crate) text: String,
    pub(crate) location: SourceLocation,
}

/// Joins backslash-continued physical lines within one source file.
///
/// WRF's `pre_parse` accumulates continuations per file, so a continuation
/// never crosses an include boundary. Callers push surviving physical lines in
/// order and must call [`LogicalLineJoiner::finish`] at end of file.
pub(crate) struct LogicalLineJoiner {
    source_name: Arc<str>,
    accumulated: String,
    start_line: usize,
    is_continuing: bool,
}

impl LogicalLineJoiner {
    pub(crate) fn new(source_name: &Arc<str>) -> Self {
        Self {
            source_name: Arc::clone(source_name),
            accumulated: String::new(),
            start_line: 1,
            is_continuing: false,
        }
    }

    /// Consumes one physical line, returning a completed logical line unless
    /// the line requests continuation.
    pub(crate) fn push(&mut self, line_number: usize, physical_line: &str) -> Option<LogicalLine> {
        let physical_line = physical_line.strip_suffix('\r').unwrap_or(physical_line);
        if !self.is_continuing {
            self.start_line = line_number;
        }

        if let Some(segment) = physical_line.strip_suffix('\\') {
            self.accumulated.push_str(segment);
            self.is_continuing = true;
            return None;
        }

        self.accumulated.push_str(physical_line);
        self.is_continuing = false;
        Some(LogicalLine {
            text: std::mem::take(&mut self.accumulated),
            location: SourceLocation::new(&self.source_name, self.start_line, 1),
        })
    }

    /// Fails when the final physical line still requests continuation.
    pub(crate) fn finish(self) -> RegistryResult<()> {
        if self.is_continuing {
            return Err(RegistryParseError::new(
                SourceLocation::new(&self.source_name, self.start_line, 1),
                RegistryParseErrorKind::DanglingContinuation,
            ));
        }
        Ok(())
    }

    pub(crate) const fn dangling_start_line(&self) -> Option<usize> {
        if self.is_continuing {
            Some(self.start_line)
        } else {
            None
        }
    }
}

pub(crate) struct LogicalLineReader;

impl LogicalLineReader {
    pub(crate) fn read(source_name: &Arc<str>, source: &str) -> RegistryResult<Vec<LogicalLine>> {
        let mut joiner = LogicalLineJoiner::new(source_name);
        let mut lines = Vec::new();

        for (line_index, physical_line) in source.lines().enumerate() {
            if let Some(line) = joiner.push(line_index + 1, physical_line) {
                lines.push(line);
            }
        }

        joiner.finish()?;
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_continuations_without_inserting_characters() {
        let source_name: Arc<str> = Arc::from("fixture");
        let lines =
            LogicalLineReader::read(&source_name, concat!("state real t \\", "\n", "ikj\n"))
                .unwrap();

        assert_eq!(lines[0].text, "state real t ikj");
        assert_eq!(lines[0].location.line(), 1);
    }

    #[test]
    fn rejects_a_continuation_at_end_of_source() {
        let source_name: Arc<str> = Arc::from("fixture");
        let error = LogicalLineReader::read(&source_name, "state real t \\").unwrap_err();

        assert_eq!(error.kind(), &RegistryParseErrorKind::DanglingContinuation);
    }

    #[test]
    fn reports_the_pending_start_line_while_a_continuation_is_open() {
        let source_name: Arc<str> = Arc::from("fixture");
        let mut joiner = LogicalLineJoiner::new(&source_name);

        assert!(joiner.push(3, "state real t \\").is_none());
        assert_eq!(joiner.dangling_start_line(), Some(3));
    }
}
