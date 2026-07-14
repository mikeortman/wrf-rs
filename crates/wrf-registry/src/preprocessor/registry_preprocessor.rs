use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::SourceLocation;
use crate::parser::logical_line::{LogicalLine, LogicalLineJoiner};
use crate::preprocessor::preprocess_error::{
    ConditionalDirective, RegistryPreprocessError, RegistryPreprocessErrorKind,
    RegistryPreprocessResult,
};
use crate::preprocessor::preprocessed_registry_source::PreprocessedRegistrySource;
use crate::preprocessor::registry_definitions::RegistryDefinitions;
use crate::preprocessor::registry_source_provider::RegistrySourceProvider;

/// Maximum number of simultaneously open sources (root plus includes).
///
/// WRF's `pre_parse` recurses without any bound, so a cyclic include never
/// terminates upstream; this crate replaces that behavior with a typed error.
const MAX_INCLUDE_DEPTH: usize = 100;

/// Maximum number of simultaneously open conditionals per file.
///
/// WRF's `pre_parse` uses a 100-slot stack whose base slot is reserved, and
/// aborts once the stack pointer reaches the end.
const MAX_CONDITIONAL_DEPTH: usize = 99;

/// Longest symbol WRF's `pre_parse` copies out of a conditional directive.
const MAX_DIRECTIVE_SYMBOL_BYTES: usize = 31;

/// Directive-like keywords the Registry language does not define.
///
/// WRF has no `else` form; upstream feeds such lines through table parsing,
/// which silently ignores them while the surrounding conditional keeps
/// selecting lines. This crate rejects them with a typed diagnostic instead.
const UNKNOWN_DIRECTIVE_KEYWORDS: [&str; 5] = ["else", "elseif", "elif", "undef", "undefine"];

/// Include and conditional expansion matching WRF's `pre_parse`.
///
/// The compatibility reference is `pre_parse` in `tools/reg_parse.c` at the
/// pinned WRF commit. Directives are recognized on raw physical lines before
/// continuation joining, comment stripping, and case folding:
///
/// - `include NAME` splices the first readable candidate from the search
///   directories, in order. Includes nest; every file keeps its own
///   conditional stack and continuation state.
/// - `ifdef SYMBOL` / `ifndef SYMBOL` select their block when the whole
///   case-sensitive symbol string (at most 31 bytes, as upstream) is defined
///   or undefined. Nested conditionals AND with their parent.
/// - `endif` closes the innermost conditional of the current file.
/// - `define SYMBOL` adds a symbol; as upstream, it takes effect even inside
///   an unselected conditional block.
///
/// Directive keywords are matched as prefixes, exactly like upstream's
/// `strncmp` checks, so `includes x` names the include file `s x`.
pub struct RegistryPreprocessor;

impl RegistryPreprocessor {
    /// Expands `root_path` into logical lines ready for parsing.
    ///
    /// Include names are joined onto each entry of `search_directories` in
    /// order; the first readable candidate wins, mirroring how upstream tries
    /// `./Registry/NAME` before `DIR/NAME`. Cycles are detected by resolved
    /// path equality; aliased paths that evade the comparison still stop at
    /// the include depth limit. Memory is proportional to the selected logical
    /// output plus the source texts in the bounded active include chain; source
    /// texts from completed includes are released as expansion unwinds.
    pub fn expand(
        root_path: impl AsRef<Path>,
        definitions: &RegistryDefinitions,
        search_directories: &[PathBuf],
        provider: &dyn RegistrySourceProvider,
    ) -> RegistryPreprocessResult<PreprocessedRegistrySource> {
        let root_path = root_path.as_ref();
        let root_name: Arc<str> = Arc::from(root_path.to_string_lossy().as_ref());
        let Some(source) = provider.read_source(root_path) else {
            return Err(RegistryPreprocessError::new(
                SourceLocation::new(&root_name, 1, 1),
                Vec::new(),
                RegistryPreprocessErrorKind::UnreadableRoot {
                    path: root_path.to_path_buf(),
                },
            ));
        };

        let mut expansion = SourceExpansion {
            provider,
            search_directories,
            definitions: definitions.clone(),
            open_paths: vec![root_path.to_path_buf()],
            inclusion_chain: Vec::new(),
            lines: Vec::new(),
        };
        expansion.expand_source(&root_name, &source)?;

        Ok(PreprocessedRegistrySource {
            root_name,
            lines: expansion.lines,
        })
    }
}

struct ConditionalFrame {
    directive: ConditionalDirective,
    symbol: String,
    /// Activity ANDed with the enclosing frame, as in upstream's stack.
    is_active: bool,
    location: SourceLocation,
}

struct SourceExpansion<'a> {
    provider: &'a dyn RegistrySourceProvider,
    search_directories: &'a [PathBuf],
    definitions: RegistryDefinitions,
    open_paths: Vec<PathBuf>,
    inclusion_chain: Vec<SourceLocation>,
    lines: Vec<LogicalLine>,
}

impl SourceExpansion<'_> {
    fn expand_source(
        &mut self,
        source_name: &Arc<str>,
        source: &str,
    ) -> RegistryPreprocessResult<()> {
        let mut conditionals: Vec<ConditionalFrame> = Vec::new();
        let mut joiner = LogicalLineJoiner::new(source_name);

        for (line_index, raw_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            let directive_text = line.trim_start_matches([' ', '\t']);
            let location = SourceLocation::new(source_name, line_number, 1);
            let is_active = conditionals.last().is_none_or(|frame| frame.is_active);

            if let Some(rest) = directive_text.strip_prefix("include") {
                if is_active {
                    self.expand_include(rest, location)?;
                }
                continue;
            }
            if let Some(rest) = directive_text.strip_prefix("ifdef") {
                self.push_conditional(
                    &mut conditionals,
                    ConditionalDirective::Ifdef,
                    rest,
                    is_active,
                    location,
                )?;
                continue;
            }
            if let Some(rest) = directive_text.strip_prefix("ifndef") {
                self.push_conditional(
                    &mut conditionals,
                    ConditionalDirective::Ifndef,
                    rest,
                    is_active,
                    location,
                )?;
                continue;
            }
            if directive_text.strip_prefix("endif").is_some() {
                if conditionals.pop().is_none() {
                    return Err(self.error(location, RegistryPreprocessErrorKind::UnmatchedEndif));
                }
                continue;
            }
            if let Some(rest) = directive_text.strip_prefix("define") {
                // Upstream defines the symbol even inside an unselected block.
                self.definitions.define(directive_symbol(rest));
                continue;
            }
            if let Some(directive) = unknown_directive_keyword(directive_text) {
                return Err(self.error(
                    location,
                    RegistryPreprocessErrorKind::UnknownDirective {
                        directive: directive.to_owned(),
                    },
                ));
            }

            if !is_active {
                continue;
            }
            if let Some(logical_line) = joiner.push(line_number, line) {
                self.lines.push(logical_line);
            }
        }

        if let Some(start_line) = joiner.dangling_start_line() {
            return Err(self.error(
                SourceLocation::new(source_name, start_line, 1),
                RegistryPreprocessErrorKind::DanglingContinuation,
            ));
        }
        if let Some(frame) = conditionals.pop() {
            return Err(self.error(
                frame.location,
                RegistryPreprocessErrorKind::UnterminatedConditional {
                    directive: frame.directive,
                    symbol: frame.symbol,
                },
            ));
        }
        Ok(())
    }

    fn push_conditional(
        &self,
        conditionals: &mut Vec<ConditionalFrame>,
        directive: ConditionalDirective,
        rest: &str,
        parent_is_active: bool,
        location: SourceLocation,
    ) -> RegistryPreprocessResult<()> {
        if conditionals.len() >= MAX_CONDITIONAL_DEPTH {
            return Err(self.error(
                location,
                RegistryPreprocessErrorKind::ConditionalDepthExceeded {
                    limit: MAX_CONDITIONAL_DEPTH,
                },
            ));
        }

        let symbol = directive_symbol(rest);
        let is_defined = self.definitions.is_defined(symbol);
        let selects = match directive {
            ConditionalDirective::Ifdef => is_defined,
            ConditionalDirective::Ifndef => !is_defined,
        };
        conditionals.push(ConditionalFrame {
            directive,
            symbol: symbol.to_owned(),
            is_active: parent_is_active && selects,
            location,
        });
        Ok(())
    }

    fn expand_include(
        &mut self,
        rest: &str,
        directive_location: SourceLocation,
    ) -> RegistryPreprocessResult<()> {
        // Upstream keeps everything after the keyword, including trailing
        // whitespace and `#` text, as part of the include file name.
        let file_name = rest.trim_start_matches([' ', '\t']);
        if file_name.is_empty() {
            return Err(self.error(
                directive_location,
                RegistryPreprocessErrorKind::EmptyIncludeName,
            ));
        }

        let mut tried_paths = Vec::new();
        let mut resolved = None;
        for directory in self.search_directories {
            let candidate = directory.join(file_name);
            match self.provider.read_source(&candidate) {
                Some(source) => {
                    resolved = Some((candidate, source));
                    break;
                }
                None => tried_paths.push(candidate),
            }
        }
        let Some((path, source)) = resolved else {
            return Err(self.error(
                directive_location,
                RegistryPreprocessErrorKind::MissingInclude {
                    file_name: file_name.to_owned(),
                    tried_paths,
                },
            ));
        };

        if self.open_paths.contains(&path) {
            return Err(self.error(
                directive_location,
                RegistryPreprocessErrorKind::CyclicInclude { path },
            ));
        }
        if self.open_paths.len() >= MAX_INCLUDE_DEPTH {
            return Err(self.error(
                directive_location,
                RegistryPreprocessErrorKind::IncludeDepthExceeded {
                    limit: MAX_INCLUDE_DEPTH,
                },
            ));
        }

        let source_name: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
        self.open_paths.push(path);
        self.inclusion_chain.push(directive_location);
        self.expand_source(&source_name, &source)?;
        self.inclusion_chain.pop();
        self.open_paths.pop();
        Ok(())
    }

    fn error(
        &self,
        location: SourceLocation,
        kind: RegistryPreprocessErrorKind,
    ) -> RegistryPreprocessError {
        RegistryPreprocessError::new(location, self.inclusion_chain.clone(), kind)
    }
}

/// Extracts a conditional or define symbol exactly as upstream `pre_parse`:
/// leading blanks skipped, at most 31 bytes copied, cut at the first blank.
fn directive_symbol(rest: &str) -> &str {
    let rest = rest.trim_start_matches([' ', '\t']);
    let mut end = rest
        .find([' ', '\t'])
        .unwrap_or(rest.len())
        .min(MAX_DIRECTIVE_SYMBOL_BYTES);
    while !rest.is_char_boundary(end) {
        end -= 1;
    }
    &rest[..end]
}

fn unknown_directive_keyword(directive_text: &str) -> Option<&str> {
    let end = directive_text
        .find([' ', '\t'])
        .unwrap_or(directive_text.len());
    let keyword = &directive_text[..end];
    UNKNOWN_DIRECTIVE_KEYWORDS
        .contains(&keyword)
        .then_some(keyword)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Write as _;
    use std::path::Path;

    use super::*;

    struct InMemorySourceProvider {
        sources_by_path: HashMap<PathBuf, String>,
    }

    impl InMemorySourceProvider {
        fn new<const N: usize>(sources: [(&str, &str); N]) -> Self {
            Self {
                sources_by_path: sources
                    .into_iter()
                    .map(|(path, text)| (PathBuf::from(path), text.to_owned()))
                    .collect(),
            }
        }
    }

    impl RegistrySourceProvider for InMemorySourceProvider {
        fn read_source(&self, path: &Path) -> Option<String> {
            self.sources_by_path.get(path).cloned()
        }
    }

    fn expand(
        provider: &InMemorySourceProvider,
        definitions: &RegistryDefinitions,
    ) -> RegistryPreprocessResult<PreprocessedRegistrySource> {
        RegistryPreprocessor::expand(
            "Registry/Registry.test",
            definitions,
            &[PathBuf::from("Registry")],
            provider,
        )
    }

    fn line_texts(source: &PreprocessedRegistrySource) -> Vec<&str> {
        source.lines().map(|(text, _)| text).collect()
    }

    #[test]
    fn splices_nested_includes_in_order_with_physical_locations() {
        let provider = InMemorySourceProvider::new([
            (
                "Registry/Registry.test",
                "first root\ninclude registry.outer\nlast root\n",
            ),
            (
                "Registry/registry.outer",
                "outer before\ninclude registry.inner\nouter after\n",
            ),
            ("Registry/registry.inner", "inner only\n"),
        ]);
        let source = expand(&provider, &RegistryDefinitions::new()).unwrap();

        assert_eq!(
            line_texts(&source),
            [
                "first root",
                "outer before",
                "inner only",
                "outer after",
                "last root",
            ]
        );
        let locations: Vec<String> = source
            .lines()
            .map(|(_, location)| location.to_string())
            .collect();
        assert_eq!(locations[0], "Registry/Registry.test:1:1");
        assert_eq!(locations[2], "Registry/registry.inner:1:1");
        assert_eq!(locations[4], "Registry/Registry.test:3:1");
    }

    #[test]
    fn selects_conditional_blocks_by_whole_symbol_string() {
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            "ifdef EM_CORE=1\nselected\nendif\nifdef EM_CORE\nrejected\nendif\n",
        )]);
        let definitions = RegistryDefinitions::from_symbols(["EM_CORE=1"]);
        let source = expand(&provider, &definitions).unwrap();

        assert_eq!(line_texts(&source), ["selected"]);
    }

    #[test]
    fn ifndef_selects_only_undefined_symbols_and_nesting_ands_with_parent() {
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            concat!(
                "ifndef MISSING\nkept\nendif\n",
                "ifndef PRESENT\ndropped\nendif\n",
                "ifdef PRESENT\nifdef MISSING\nalso dropped\nendif\nstill kept\nendif\n",
            ),
        )]);
        let definitions = RegistryDefinitions::from_symbols(["PRESENT"]);
        let source = expand(&provider, &definitions).unwrap();

        assert_eq!(line_texts(&source), ["kept", "still kept"]);
    }

    #[test]
    fn define_takes_effect_even_inside_an_unselected_block() {
        // Upstream quirk: the define branch in pre_parse never consults the
        // ifdef stack, so unselected blocks still define symbols.
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            "ifdef MISSING\ndefine LEAKED\nendif\nifdef LEAKED\nleaked line\nendif\n",
        )]);
        let source = expand(&provider, &RegistryDefinitions::new()).unwrap();

        assert_eq!(line_texts(&source), ["leaked line"]);
    }

    #[test]
    fn truncates_directive_symbols_to_thirty_one_bytes() {
        let long_symbol = "S".repeat(40);
        let mut root = String::new();
        writeln!(root, "define {long_symbol}").unwrap();
        writeln!(root, "ifdef {}", &long_symbol[..31]).unwrap();
        root.push_str("kept\nendif\n");
        let provider = InMemorySourceProvider::new([("Registry/Registry.test", root.as_str())]);
        let source = expand(&provider, &RegistryDefinitions::new()).unwrap();

        assert_eq!(line_texts(&source), ["kept"]);
    }

    #[test]
    fn conditionals_inside_an_include_do_not_leak_into_the_parent() {
        let provider = InMemorySourceProvider::new([
            (
                "Registry/Registry.test",
                "include registry.guarded\nafter include\n",
            ),
            (
                "Registry/registry.guarded",
                "ifdef MISSING\nhidden\nendif\nvisible\n",
            ),
        ]);
        let source = expand(&provider, &RegistryDefinitions::new()).unwrap();

        assert_eq!(line_texts(&source), ["visible", "after include"]);
    }

    #[test]
    fn skips_includes_inside_unselected_blocks_without_resolving_them() {
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            "ifdef MISSING\ninclude registry.absent\nendif\nkept\n",
        )]);
        let source = expand(&provider, &RegistryDefinitions::new()).unwrap();

        assert_eq!(line_texts(&source), ["kept"]);
    }

    #[test]
    fn keeps_continuations_within_one_file_across_directive_lines() {
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            "state real t \\\nifdef MISSING\ndropped\nendif\nikj\n",
        )]);
        let source = expand(&provider, &RegistryDefinitions::new()).unwrap();

        assert_eq!(line_texts(&source), ["state real t ikj"]);
    }

    #[test]
    fn reports_a_missing_include_with_tried_paths_and_chain() {
        let provider = InMemorySourceProvider::new([
            ("Registry/Registry.test", "include registry.outer\n"),
            ("Registry/registry.outer", "include registry.absent\n"),
        ]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.location().to_string(), "Registry/registry.outer:1:1");
        assert_eq!(error.inclusion_chain().len(), 1);
        assert_eq!(
            error.inclusion_chain()[0].to_string(),
            "Registry/Registry.test:1:1"
        );
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::MissingInclude { file_name, tried_paths }
                if file_name == "registry.absent" && tried_paths.len() == 1
        ));
    }

    #[test]
    fn rejects_a_self_include_as_cyclic() {
        let provider =
            InMemorySourceProvider::new([("Registry/Registry.test", "include Registry.test\n")]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::CyclicInclude { path }
                if path == Path::new("Registry/Registry.test")
        ));
    }

    #[test]
    fn rejects_an_indirect_include_cycle_with_the_full_chain() {
        let provider = InMemorySourceProvider::new([
            ("Registry/Registry.test", "include registry.a\n"),
            ("Registry/registry.a", "include registry.b\n"),
            ("Registry/registry.b", "include registry.a\n"),
        ]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.location().to_string(), "Registry/registry.b:1:1");
        assert_eq!(error.inclusion_chain().len(), 2);
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::CyclicInclude { path }
                if path == Path::new("Registry/registry.a")
        ));
    }

    #[test]
    fn stops_non_cyclic_include_chains_at_the_depth_limit() {
        let mut sources = vec![(
            "Registry/Registry.test".to_owned(),
            "include registry.depth.1\n".to_owned(),
        )];
        for depth in 1..=MAX_INCLUDE_DEPTH {
            sources.push((
                format!("Registry/registry.depth.{depth}"),
                format!("include registry.depth.{}\n", depth + 1),
            ));
        }
        let provider = InMemorySourceProvider {
            sources_by_path: sources
                .into_iter()
                .map(|(path, text)| (PathBuf::from(path), text))
                .collect(),
        };
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(
            error.kind(),
            &RegistryPreprocessErrorKind::IncludeDepthExceeded {
                limit: MAX_INCLUDE_DEPTH
            }
        );
        assert_eq!(error.inclusion_chain().len(), MAX_INCLUDE_DEPTH - 1);
    }

    #[test]
    fn stops_conditional_nesting_at_the_upstream_stack_limit() {
        let mut root = String::new();
        for _ in 0..=MAX_CONDITIONAL_DEPTH {
            root.push_str("ifdef MISSING\n");
        }
        for _ in 0..=MAX_CONDITIONAL_DEPTH {
            root.push_str("endif\n");
        }
        let provider = InMemorySourceProvider::new([("Registry/Registry.test", root.as_str())]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.location().line(), MAX_CONDITIONAL_DEPTH + 1);
        assert_eq!(
            error.kind(),
            &RegistryPreprocessErrorKind::ConditionalDepthExceeded {
                limit: MAX_CONDITIONAL_DEPTH
            }
        );
    }

    #[test]
    fn rejects_an_endif_without_an_open_conditional() {
        let provider = InMemorySourceProvider::new([("Registry/Registry.test", "kept\nendif\n")]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.location().line(), 2);
        assert_eq!(error.kind(), &RegistryPreprocessErrorKind::UnmatchedEndif);
    }

    #[test]
    fn reports_an_unterminated_conditional_at_its_opening_line() {
        let provider = InMemorySourceProvider::new([
            ("Registry/Registry.test", "include registry.open\n"),
            ("Registry/registry.open", "kept\nifdef PRESENT\nbody\n"),
        ]);
        let definitions = RegistryDefinitions::from_symbols(["PRESENT"]);
        let error = expand(&provider, &definitions).unwrap_err();

        assert_eq!(error.location().to_string(), "Registry/registry.open:2:1");
        assert_eq!(error.inclusion_chain().len(), 1);
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::UnterminatedConditional { directive, symbol }
                if *directive == ConditionalDirective::Ifdef && symbol == "PRESENT"
        ));
    }

    #[test]
    fn rejects_else_style_directives_the_language_does_not_define() {
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            "ifdef MISSING\nhidden\nelse\nvisible\nendif\n",
        )]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.location().line(), 3);
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::UnknownDirective { directive } if directive == "else"
        ));
    }

    #[test]
    fn rejects_a_dangling_continuation_inside_an_included_file() {
        let provider = InMemorySourceProvider::new([
            ("Registry/Registry.test", "include registry.tail\n"),
            ("Registry/registry.tail", "state real t \\"),
        ]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.location().to_string(), "Registry/registry.tail:1:1");
        assert_eq!(
            error.kind(),
            &RegistryPreprocessErrorKind::DanglingContinuation
        );
    }

    #[test]
    fn rejects_an_include_directive_without_a_file_name() {
        let provider = InMemorySourceProvider::new([("Registry/Registry.test", "include\n")]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert_eq!(error.kind(), &RegistryPreprocessErrorKind::EmptyIncludeName);
    }

    #[test]
    fn reports_an_unreadable_root_source() {
        let provider = InMemorySourceProvider::new([]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::UnreadableRoot { path }
                if path == Path::new("Registry/Registry.test")
        ));
    }

    #[test]
    fn keeps_upstream_prefix_matching_and_raw_include_names() {
        // Upstream matches directives with strncmp and keeps `#` text as part
        // of the include name, so this include resolves the literal name
        // `registry.extra # trailing`.
        let provider = InMemorySourceProvider::new([(
            "Registry/Registry.test",
            "include registry.extra # trailing\n",
        )]);
        let error = expand(&provider, &RegistryDefinitions::new()).unwrap_err();

        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::MissingInclude { file_name, .. }
                if file_name == "registry.extra # trailing"
        ));
    }
}
