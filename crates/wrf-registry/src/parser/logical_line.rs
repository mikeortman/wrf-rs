use std::sync::Arc;

use crate::{RegistryParseError, RegistryParseErrorKind, RegistryResult, SourceLocation};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct LogicalLine {
    pub(super) text: String,
    pub(super) location: SourceLocation,
}

pub(super) struct LogicalLineReader;

impl LogicalLineReader {
    pub(super) fn read(source_name: &Arc<str>, source: &str) -> RegistryResult<Vec<LogicalLine>> {
        let mut lines = Vec::new();
        let mut accumulated = String::new();
        let mut start_line = 1;
        let mut is_continuing = false;

        for (line_index, physical_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let physical_line = physical_line.strip_suffix('\r').unwrap_or(physical_line);

            if !is_continuing {
                start_line = line_number;
            }

            if let Some(segment) = physical_line.strip_suffix('\\') {
                accumulated.push_str(segment);
                is_continuing = true;
                continue;
            }

            accumulated.push_str(physical_line);
            lines.push(LogicalLine {
                text: std::mem::take(&mut accumulated),
                location: SourceLocation::new(source_name, start_line, 1),
            });
            is_continuing = false;
        }

        if is_continuing {
            return Err(RegistryParseError::new(
                SourceLocation::new(source_name, start_line, 1),
                RegistryParseErrorKind::DanglingContinuation,
            ));
        }

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
}
