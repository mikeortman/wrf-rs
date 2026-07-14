use std::fmt;
use std::sync::Arc;

/// One-based source location retained from a Registry input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    source_name: Arc<str>,
    line: usize,
    column: usize,
}

impl SourceLocation {
    pub(crate) fn new(source_name: &Arc<str>, line: usize, column: usize) -> Self {
        Self {
            source_name: Arc::clone(source_name),
            line,
            column,
        }
    }

    /// Returns the caller-supplied source name.
    #[must_use]
    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    /// Returns the one-based physical line number.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the one-based physical column number.
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }

    pub(crate) fn with_column(&self, column: usize) -> Self {
        Self {
            source_name: Arc::clone(&self.source_name),
            line: self.line,
            column,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}",
            self.source_name, self.line, self.column
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_conventional_source_location() {
        let source_name: Arc<str> = Arc::from("Registry.EM");
        let location = SourceLocation::new(&source_name, 17, 4);

        assert_eq!(location.to_string(), "Registry.EM:17:4");
    }
}
