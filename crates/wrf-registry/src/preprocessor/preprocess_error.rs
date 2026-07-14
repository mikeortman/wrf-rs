use std::fmt;
use std::path::PathBuf;

use crate::SourceLocation;

/// Conditional directive keyword recorded in preprocessing diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConditionalDirective {
    /// `ifdef SYMBOL` selects its block when `SYMBOL` is defined.
    Ifdef,
    /// `ifndef SYMBOL` selects its block when `SYMBOL` is not defined.
    Ifndef,
}

impl ConditionalDirective {
    /// Returns the directive keyword as written in Registry sources.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Ifdef => "ifdef",
            Self::Ifndef => "ifndef",
        }
    }
}

/// Specific reason Registry preprocessing failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryPreprocessErrorKind {
    /// The root Registry source could not be read.
    UnreadableRoot {
        /// Root path handed to the preprocessor.
        path: PathBuf,
    },
    /// An `include` directive has no file name.
    EmptyIncludeName,
    /// No search directory yielded a readable include file.
    MissingInclude {
        /// Include name exactly as written after the directive keyword.
        file_name: String,
        /// Candidate paths tried in search order.
        tried_paths: Vec<PathBuf>,
    },
    /// An include resolves to a file that is already being expanded.
    CyclicInclude {
        /// Resolved path that closed the cycle.
        path: PathBuf,
    },
    /// Include nesting exceeds the supported depth.
    IncludeDepthExceeded {
        /// Maximum number of simultaneously open sources.
        limit: usize,
    },
    /// Conditional nesting exceeds WRF's 100-slot `ifdef` stack.
    ConditionalDepthExceeded {
        /// Maximum number of simultaneously open conditionals.
        limit: usize,
    },
    /// An `endif` appears without an open conditional in the same file.
    UnmatchedEndif,
    /// A conditional block is still open at the end of its file.
    UnterminatedConditional {
        /// Directive that opened the block.
        directive: ConditionalDirective,
        /// Symbol the open block tests.
        symbol: String,
    },
    /// A directive-like keyword the Registry language does not define.
    UnknownDirective {
        /// Offending keyword.
        directive: String,
    },
    /// The final physical line of a file still requests continuation.
    DanglingContinuation,
}

/// Typed preprocessing failure with its physical location and include chain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryPreprocessError {
    location: SourceLocation,
    inclusion_chain: Vec<SourceLocation>,
    kind: RegistryPreprocessErrorKind,
}

/// Result returned by Registry preprocessing operations.
pub type RegistryPreprocessResult<T> = Result<T, RegistryPreprocessError>;

impl RegistryPreprocessError {
    pub(crate) const fn new(
        location: SourceLocation,
        inclusion_chain: Vec<SourceLocation>,
        kind: RegistryPreprocessErrorKind,
    ) -> Self {
        Self {
            location,
            inclusion_chain,
            kind,
        }
    }

    /// Returns the physical location of the offending line.
    ///
    /// For [`RegistryPreprocessErrorKind::UnterminatedConditional`] this is
    /// the location of the directive that opened the block.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the `include` directive locations from the root source down to
    /// the file containing [`RegistryPreprocessError::location`], in order.
    ///
    /// The chain is empty when the failure is in the root source itself.
    #[must_use]
    pub fn inclusion_chain(&self) -> &[SourceLocation] {
        &self.inclusion_chain
    }

    /// Returns the typed reason preprocessing failed.
    #[must_use]
    pub const fn kind(&self) -> &RegistryPreprocessErrorKind {
        &self.kind
    }
}

impl fmt::Display for RegistryPreprocessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: ", self.location)?;
        match &self.kind {
            RegistryPreprocessErrorKind::UnreadableRoot { path } => {
                write!(
                    formatter,
                    "cannot read Registry source `{}`",
                    path.display()
                )
            }
            RegistryPreprocessErrorKind::EmptyIncludeName => {
                formatter.write_str("include directive has no file name")
            }
            RegistryPreprocessErrorKind::MissingInclude {
                file_name,
                tried_paths,
            } => {
                write!(formatter, "cannot open include `{file_name}`; tried")?;
                for path in tried_paths {
                    write!(formatter, " `{}`", path.display())?;
                }
                Ok(())
            }
            RegistryPreprocessErrorKind::CyclicInclude { path } => {
                write!(
                    formatter,
                    "include `{}` is already being expanded",
                    path.display()
                )
            }
            RegistryPreprocessErrorKind::IncludeDepthExceeded { limit } => {
                write!(formatter, "include nesting exceeds {limit} open sources")
            }
            RegistryPreprocessErrorKind::ConditionalDepthExceeded { limit } => {
                write!(formatter, "conditional nesting exceeds {limit} open blocks")
            }
            RegistryPreprocessErrorKind::UnmatchedEndif => {
                formatter.write_str("endif without an open conditional")
            }
            RegistryPreprocessErrorKind::UnterminatedConditional { directive, symbol } => {
                write!(
                    formatter,
                    "{} {symbol} is still open at end of file",
                    directive.keyword()
                )
            }
            RegistryPreprocessErrorKind::UnknownDirective { directive } => {
                write!(formatter, "unknown preprocessing directive `{directive}`")
            }
            RegistryPreprocessErrorKind::DanglingContinuation => {
                formatter.write_str("backslash continuation reaches end of file")
            }
        }?;

        for include_location in self.inclusion_chain.iter().rev() {
            write!(formatter, "\n  included from {include_location}")?;
        }
        Ok(())
    }
}

impl std::error::Error for RegistryPreprocessError {}
