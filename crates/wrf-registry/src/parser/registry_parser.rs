use std::collections::HashSet;
use std::num::NonZeroU8;
use std::path::Path;
use std::sync::Arc;

use crate::model::{
    ConfigurationEntryCount, CoordinateAxis, DimensionLength, DimensionSpecification,
    ProcessorOrientation, RegistryDocument, RegistryEntry, RegistryValueType, RuntimeConfiguration,
    StateDimensions, StateStaggering, StateVariable,
};
use crate::parser::logical_line::{LogicalLine, LogicalLineReader};
use crate::parser::tokenizer::RegistryTokenizer;
use crate::preprocessor::{
    FileSystemSourceProvider, PreprocessedRegistrySource, RegistryDefinitions, RegistryPreprocessor,
};
use crate::{
    RegistryParseError, RegistryParseErrorKind, RegistryResult, RegistrySourceError, SourceLocation,
};

/// Parser for the dependency-closed WRF Registry subset documented by this crate.
pub struct RegistryParser;

impl RegistryParser {
    /// Parses one Registry source while retaining physical source locations.
    ///
    /// The source must already be preprocessed: `include` and conditional
    /// directives are rejected as unsupported entries. Use
    /// [`RegistryParser::parse_file`] or [`RegistryParser::parse_preprocessed`]
    /// for sources that rely on preprocessing. Dimensions must appear before
    /// states that reference them. The returned document owns its strings and
    /// does not borrow from `source`.
    pub fn parse(
        source_name: impl Into<Arc<str>>,
        source: &str,
    ) -> RegistryResult<RegistryDocument> {
        let source_name = source_name.into();
        let logical_lines = LogicalLineReader::read(&source_name, source)?;
        Self::parse_logical_lines(source_name, &logical_lines)
    }

    /// Expands includes and conditionals from `root_path`, then parses.
    ///
    /// Include names are resolved like WRF's `registry` program: first against
    /// `./Registry/` relative to the current working directory, then against
    /// the directory containing `root_path`. Entry locations keep the physical
    /// file and line the entry came from, across nested includes.
    pub fn parse_file(
        root_path: impl AsRef<Path>,
        definitions: &RegistryDefinitions,
    ) -> Result<RegistryDocument, RegistrySourceError> {
        let root_path = root_path.as_ref();
        let root_directory = root_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map_or_else(|| Path::new(".").to_path_buf(), Path::to_path_buf);
        let search_directories = [Path::new("./Registry").to_path_buf(), root_directory];
        let preprocessed = RegistryPreprocessor::expand(
            root_path,
            definitions,
            &search_directories,
            &FileSystemSourceProvider,
        )?;
        Ok(Self::parse_preprocessed(&preprocessed)?)
    }

    /// Parses an already include-expanded and conditional-filtered source.
    pub fn parse_preprocessed(
        source: &PreprocessedRegistrySource,
    ) -> RegistryResult<RegistryDocument> {
        Self::parse_logical_lines(Arc::clone(&source.root_name), &source.lines)
    }

    fn parse_logical_lines(
        source_name: Arc<str>,
        logical_lines: &[LogicalLine],
    ) -> RegistryResult<RegistryDocument> {
        let mut dimension_names = HashSet::new();
        let mut entries = Vec::new();

        for logical_line in logical_lines {
            let tokens = RegistryTokenizer::tokenize(&logical_line.location, &logical_line.text)?;
            if tokens.is_empty() {
                continue;
            }

            let entry = Self::parse_entry(&tokens, logical_line, &mut dimension_names)?;
            entries.push(entry);
        }

        Ok(RegistryDocument {
            source_name,
            entries,
        })
    }

    fn parse_entry(
        tokens: &[String],
        line: &LogicalLine,
        dimension_names: &mut HashSet<String>,
    ) -> RegistryResult<RegistryEntry> {
        let location = line.location.clone();
        match tokens[0].as_str() {
            "dimspec" => Self::parse_dimension(tokens, location, dimension_names)
                .map(RegistryEntry::Dimension),
            "state" => {
                Self::parse_state(tokens, location, dimension_names).map(RegistryEntry::State)
            }
            "rconfig" => Self::parse_runtime_configuration(tokens, location)
                .map(RegistryEntry::RuntimeConfiguration),
            entry_kind => Err(RegistryParseError::new(
                location,
                RegistryParseErrorKind::UnsupportedEntry {
                    entry_kind: entry_kind.to_owned(),
                },
            )),
        }
    }

    fn parse_dimension(
        tokens: &[String],
        location: SourceLocation,
        dimension_names: &mut HashSet<String>,
    ) -> RegistryResult<DimensionSpecification> {
        Self::require_token_count(tokens, 6, &location)?;
        let name = tokens[1].clone();
        if !dimension_names.insert(name.clone()) {
            return Err(RegistryParseError::new(
                location,
                RegistryParseErrorKind::DuplicateDimension { name },
            ));
        }

        let order = Self::parse_dimension_order(&tokens[2], &location)?;
        let length = Self::parse_dimension_length(&tokens[3], &location)?;
        if matches!(length, DimensionLength::StandardDomain) && !matches!(order, Some(1..=3)) {
            return Err(RegistryParseError::new(
                location,
                RegistryParseErrorKind::InvalidDimensionOrder {
                    value: tokens[2].clone(),
                },
            ));
        }

        let axis = match tokens[4].as_str() {
            "x" => CoordinateAxis::X,
            "y" => CoordinateAxis::Y,
            "z" => CoordinateAxis::Z,
            "c" | "-" => CoordinateAxis::Constant,
            _ => {
                return Err(RegistryParseError::new(
                    location,
                    RegistryParseErrorKind::InvalidCoordinateAxis {
                        value: tokens[4].clone(),
                    },
                ));
            }
        };

        Ok(DimensionSpecification {
            location,
            name,
            order,
            length,
            axis,
            data_name: Self::optional_token(&tokens[5]),
        })
    }

    fn parse_state(
        tokens: &[String],
        location: SourceLocation,
        dimension_names: &HashSet<String>,
    ) -> RegistryResult<StateVariable> {
        Self::require_token_count(tokens, 11, &location)?;
        let value_type = Self::parse_value_type(&tokens[1], &location)?;
        let dimensions = Self::parse_state_dimensions(&tokens[3], dimension_names, &location)?;
        let time_levels = if tokens[5] == "-" {
            NonZeroU8::MIN
        } else {
            tokens[5]
                .parse::<u8>()
                .ok()
                .and_then(NonZeroU8::new)
                .ok_or_else(|| {
                    RegistryParseError::new(
                        location.clone(),
                        RegistryParseErrorKind::InvalidTimeLevels {
                            value: tokens[5].clone(),
                        },
                    )
                })?
        };
        let staggering = Self::parse_staggering(&tokens[6], &location)?;

        Ok(StateVariable {
            location,
            value_type,
            name: tokens[2].clone(),
            dimensions,
            use_association: Self::optional_token(&tokens[4]),
            time_levels,
            staggering,
            io_specification: Self::optional_token(&tokens[7]),
            data_name: Self::optional_token(&tokens[8]),
            description: Self::optional_token(&tokens[9]),
            units: Self::optional_token(&tokens[10]),
        })
    }

    fn parse_runtime_configuration(
        tokens: &[String],
        location: SourceLocation,
    ) -> RegistryResult<RuntimeConfiguration> {
        Self::require_token_count(tokens, 10, &location)?;
        let entry_count = match tokens[4].as_str() {
            "-" | "1" => ConfigurationEntryCount::Scalar,
            expression => ConfigurationEntryCount::Expression(expression.to_owned()),
        };

        Ok(RuntimeConfiguration {
            location: location.clone(),
            value_type: Self::parse_value_type(&tokens[1], &location)?,
            name: tokens[2].clone(),
            how_set: Self::optional_token(&tokens[3]),
            entry_count,
            default_value: Self::optional_token(&tokens[5]),
            io_specification: Self::optional_token(&tokens[6]),
            data_name: Self::optional_token(&tokens[7]),
            description: Self::optional_token(&tokens[8]),
            units: Self::optional_token(&tokens[9]),
        })
    }

    fn parse_dimension_order(token: &str, location: &SourceLocation) -> RegistryResult<Option<u8>> {
        if token == "-" {
            return Ok(None);
        }
        token.parse::<u8>().map(Some).map_err(|_| {
            RegistryParseError::new(
                location.clone(),
                RegistryParseErrorKind::InvalidDimensionOrder {
                    value: token.to_owned(),
                },
            )
        })
    }

    fn parse_dimension_length(
        token: &str,
        location: &SourceLocation,
    ) -> RegistryResult<DimensionLength> {
        if token == "standard_domain" {
            return Ok(DimensionLength::StandardDomain);
        }

        if let Some(value) = token.strip_prefix("constant=") {
            return Self::parse_constant_dimension(value, token, location);
        }

        if let Some(value) = token.strip_prefix("namelist=") {
            if value.is_empty() {
                return Self::invalid_dimension_length(token, location);
            }
            let (start, end) = value
                .split_once(':')
                .map_or_else(|| ("1", value), |(start, end)| (start, end));
            if start.is_empty() || end.is_empty() {
                return Self::invalid_dimension_length(token, location);
            }
            return Ok(DimensionLength::Namelist {
                start: start.to_owned(),
                end: end.to_owned(),
            });
        }

        Self::invalid_dimension_length(token, location)
    }

    fn parse_constant_dimension(
        value: &str,
        original_token: &str,
        location: &SourceLocation,
    ) -> RegistryResult<DimensionLength> {
        let bounds = if value.starts_with('(') && value.ends_with(')') {
            &value[1..value.len() - 1]
        } else if value.contains(':') {
            return Self::invalid_dimension_length(original_token, location);
        } else {
            let end = value.parse::<i32>().map_err(|_| {
                RegistryParseError::new(
                    location.clone(),
                    RegistryParseErrorKind::InvalidDimensionLength {
                        value: original_token.to_owned(),
                    },
                )
            })?;
            return Ok(DimensionLength::Constant { start: 1, end });
        };

        let Some((start, end)) = bounds.split_once(':') else {
            return Self::invalid_dimension_length(original_token, location);
        };
        let Ok(start) = start.parse::<i32>() else {
            return Self::invalid_dimension_length(original_token, location);
        };
        let Ok(end) = end.parse::<i32>() else {
            return Self::invalid_dimension_length(original_token, location);
        };
        Ok(DimensionLength::Constant { start, end })
    }

    fn parse_state_dimensions(
        token: &str,
        dimension_names: &HashSet<String>,
        location: &SourceLocation,
    ) -> RegistryResult<StateDimensions> {
        if token == "-" {
            return Ok(StateDimensions {
                names: Vec::new(),
                subgrid_positions: Vec::new(),
                processor_orientation: ProcessorOrientation::Z,
                is_boundary_array: false,
                is_scalar_array_member: false,
                has_scalar_array_tendencies: false,
            });
        }
        if token.contains(['{', '}']) {
            return Err(RegistryParseError::new(
                location.clone(),
                RegistryParseErrorKind::UnsupportedStateDimensionSyntax {
                    value: token.to_owned(),
                },
            ));
        }

        let mut dimensions = StateDimensions {
            names: Vec::new(),
            subgrid_positions: Vec::new(),
            processor_orientation: ProcessorOrientation::Z,
            is_boundary_array: false,
            is_scalar_array_member: false,
            has_scalar_array_tendencies: false,
        };
        let mut next_is_subgrid = false;
        let mut saw_modifier = false;

        for character in token.chars() {
            if character == '*' && !saw_modifier {
                if next_is_subgrid {
                    return Self::invalid_state_dimensions(token, location);
                }
                next_is_subgrid = true;
                continue;
            }

            if matches!(character, 'f' | 't' | 'x' | 'y' | 'b') {
                saw_modifier = true;
                match character {
                    'f' => dimensions.is_scalar_array_member = true,
                    't' => dimensions.has_scalar_array_tendencies = true,
                    'x' => dimensions.processor_orientation = ProcessorOrientation::X,
                    'y' => dimensions.processor_orientation = ProcessorOrientation::Y,
                    'b' => dimensions.is_boundary_array = true,
                    _ => unreachable!(),
                }
                continue;
            }

            if saw_modifier {
                return Self::invalid_state_dimensions(token, location);
            }

            let name = character.to_string();
            if !dimension_names.contains(&name) {
                return Err(RegistryParseError::new(
                    location.clone(),
                    RegistryParseErrorKind::UnknownDimension { name },
                ));
            }
            if next_is_subgrid {
                dimensions.subgrid_positions.push(dimensions.names.len());
                next_is_subgrid = false;
            }
            dimensions.names.push(name);
        }

        if next_is_subgrid {
            return Self::invalid_state_dimensions(token, location);
        }
        Ok(dimensions)
    }

    fn parse_staggering(token: &str, location: &SourceLocation) -> RegistryResult<StateStaggering> {
        if token == "-" {
            return Ok(StateStaggering::default());
        }
        let mut staggering = StateStaggering::default();
        for flag in token.chars() {
            match flag {
                'x' => staggering.x = true,
                'y' => staggering.y = true,
                'z' => staggering.z = true,
                'v' => staggering.nmm_vertical_grid = true,
                'm' => staggering.microphysics_variable = true,
                'f' => staggering.full_feedback = true,
                'n' => staggering.no_feedback = true,
                _ => {
                    return Err(RegistryParseError::new(
                        location.clone(),
                        RegistryParseErrorKind::InvalidStaggering {
                            value: token.to_owned(),
                        },
                    ));
                }
            }
        }
        Ok(staggering)
    }

    fn parse_value_type(
        token: &str,
        location: &SourceLocation,
    ) -> RegistryResult<RegistryValueType> {
        match token {
            "integer" => Ok(RegistryValueType::Integer),
            "real" => Ok(RegistryValueType::Real),
            "logical" => Ok(RegistryValueType::Logical),
            "character*256" => Ok(RegistryValueType::Character256),
            "double" | "doubleprecision" => Ok(RegistryValueType::DoublePrecision),
            _ => Err(RegistryParseError::new(
                location.clone(),
                RegistryParseErrorKind::InvalidValueType {
                    value: token.to_owned(),
                },
            )),
        }
    }

    fn require_token_count(
        tokens: &[String],
        expected: usize,
        location: &SourceLocation,
    ) -> RegistryResult<()> {
        if tokens.len() == expected {
            return Ok(());
        }
        Err(RegistryParseError::new(
            location.clone(),
            RegistryParseErrorKind::UnexpectedTokenCount {
                entry_kind: tokens[0].clone(),
                expected,
                actual: tokens.len(),
            },
        ))
    }

    fn optional_token(token: &str) -> Option<String> {
        (token != "-").then(|| token.to_owned())
    }

    fn invalid_dimension_length<T>(token: &str, location: &SourceLocation) -> RegistryResult<T> {
        Err(RegistryParseError::new(
            location.clone(),
            RegistryParseErrorKind::InvalidDimensionLength {
                value: token.to_owned(),
            },
        ))
    }

    fn invalid_state_dimensions<T>(token: &str, location: &SourceLocation) -> RegistryResult<T> {
        Err(RegistryParseError::new(
            location.clone(),
            RegistryParseErrorKind::UnsupportedStateDimensionSyntax {
                value: token.to_owned(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::preprocessor::{RegistryPreprocessError, RegistryPreprocessErrorKind};

    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../parity/registry/fixtures")
            .join(name)
    }

    fn parity_definitions() -> RegistryDefinitions {
        RegistryDefinitions::from_symbols(["PARITY_SLICE=1"])
    }

    fn preprocess_error(name: &str) -> RegistryPreprocessError {
        match RegistryParser::parse_file(fixture_path(name), &parity_definitions()).unwrap_err() {
            RegistrySourceError::Preprocess(error) => error,
            RegistrySourceError::Parse(error) => {
                panic!("expected a preprocess error, found parse error {error}")
            }
        }
    }

    #[test]
    fn parses_the_arw_slice_across_nested_includes_and_conditionals() {
        let document =
            RegistryParser::parse_file(fixture_path("registry_arw_slice"), &parity_definitions())
                .unwrap();
        let dimensions: Vec<_> = document.dimensions().collect();
        let states: Vec<_> = document.state_variables().collect();
        let configurations: Vec<_> = document.runtime_configurations().collect();

        assert_eq!(dimensions.len(), 6);
        assert_eq!(states.len(), 3);
        assert_eq!(configurations.len(), 7);
        assert_eq!(
            dimensions[4].length(),
            &DimensionLength::Constant { start: 1, end: 12 }
        );
        assert!(states[0].dimensions().is_boundary_array());
        assert_eq!(states[0].dimensions().names(), &["i", "k", "j"]);
        assert_eq!(states[0].time_levels().get(), 2);
        assert_eq!(
            configurations[4].description(),
            Some("Case-preserving input path")
        );

        // Entries keep the physical file they came from through the includes.
        assert!(
            dimensions[0]
                .location()
                .source_name()
                .ends_with("registry.dimspec_slice")
        );
        assert!(
            states[0]
                .location()
                .source_name()
                .ends_with("registry.state_slice")
        );
        assert!(
            states[2]
                .location()
                .source_name()
                .ends_with("registry_arw_slice")
        );
        assert_eq!(states[2].name(), "qv_slice");
        assert_eq!(configurations[5].name(), "parity_slice_opt");
        assert_eq!(configurations[6].name(), "unset_feature_default");
    }

    #[test]
    fn excludes_every_entry_guarded_by_an_undefined_symbol() {
        let document =
            RegistryParser::parse_file(fixture_path("registry_arw_slice"), &parity_definitions())
                .unwrap();

        assert!(
            document
                .state_variables()
                .all(|state| state.name() != "ghost")
        );
        assert!(
            document
                .runtime_configurations()
                .all(|configuration| !matches!(configuration.name(), "ghost_opt" | "ghost_ifndef"))
        );
    }

    #[test]
    fn selects_ifndef_entries_when_no_symbols_are_defined() {
        let document = RegistryParser::parse_file(
            fixture_path("registry_arw_slice"),
            &RegistryDefinitions::new(),
        )
        .unwrap();

        assert_eq!(document.state_variables().count(), 2);
        let configuration_names: Vec<_> = document
            .runtime_configurations()
            .map(|configuration| configuration.name())
            .collect();
        assert_eq!(configuration_names.len(), 7);
        assert!(configuration_names.contains(&"unset_feature_default"));
        assert!(configuration_names.contains(&"ghost_ifndef"));
        assert!(!configuration_names.contains(&"parity_slice_opt"));
    }

    #[test]
    fn reports_a_missing_include_file_with_its_candidates() {
        let error = preprocess_error("malformed/registry_missing_include");

        assert_eq!(error.location().line(), 2);
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::MissingInclude { file_name, tried_paths }
                if file_name == "registry.does_not_exist" && tried_paths.len() == 2
        ));
    }

    #[test]
    fn reports_a_self_include_as_cyclic() {
        let error = preprocess_error("malformed/registry_self_include");

        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::CyclicInclude { .. }
        ));
    }

    #[test]
    fn reports_an_indirect_include_cycle_with_its_chain() {
        let error = preprocess_error("malformed/registry_cycle_a");

        assert_eq!(error.inclusion_chain().len(), 1);
        assert!(error.location().source_name().ends_with("registry_cycle_b"));
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::CyclicInclude { .. }
        ));
    }

    #[test]
    fn reports_an_unterminated_conditional_at_its_opening_line() {
        let error = preprocess_error("malformed/registry_unterminated_conditional");

        assert_eq!(error.location().line(), 3);
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::UnterminatedConditional { symbol, .. }
                if symbol == "PARITY_SLICE=1"
        ));
    }

    #[test]
    fn reports_an_unknown_else_directive() {
        let error = preprocess_error("malformed/registry_unknown_directive");

        assert_eq!(error.location().line(), 4);
        assert!(matches!(
            error.kind(),
            RegistryPreprocessErrorKind::UnknownDirective { directive } if directive == "else"
        ));
    }

    #[test]
    fn reports_a_parse_failure_at_its_physical_nested_include_location() {
        let error = RegistryParser::parse_file(
            fixture_path("malformed/registry_bad_nested_entry"),
            &parity_definitions(),
        )
        .unwrap_err();

        let RegistrySourceError::Parse(error) = error else {
            panic!("expected a parse error from the included source");
        };
        assert!(
            error
                .location()
                .source_name()
                .ends_with("registry_bad_nested_entry_child")
        );
        assert_eq!(error.location().line(), 2);
        assert_eq!(error.kind(), &RegistryParseErrorKind::UnbalancedQuote);
    }

    #[test]
    fn preprocessing_failures_leave_no_partial_document() {
        // Failure atomicity: parse_file returns only Err, never a partially
        // populated document, when any included file is malformed.
        let result = RegistryParser::parse_file(
            fixture_path("malformed/registry_missing_include"),
            &parity_definitions(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn reports_the_first_physical_line_of_a_malformed_continuation() {
        let source = concat!(
            "dimspec i 1 standard_domain x west_east\n",
            "state real t \\",
            "\n",
            "i dyn_em nope - irh T \"temperature\" \"K\"\n"
        );
        let error = RegistryParser::parse("Registry.bad", source).unwrap_err();

        assert_eq!(error.location().line(), 2);
        assert!(matches!(
            error.kind(),
            RegistryParseErrorKind::InvalidTimeLevels { value } if value == "nope"
        ));
    }

    #[test]
    fn rejects_state_dimensions_defined_after_use() {
        let source = "state real t i dyn_em 1 - irh T temperature K\n";
        let error = RegistryParser::parse("Registry.bad", source).unwrap_err();

        assert_eq!(
            error.kind(),
            &RegistryParseErrorKind::UnknownDimension {
                name: "i".to_owned()
            }
        );
    }

    #[test]
    fn rejects_duplicate_dimensions() {
        let source = "dimspec i 1 standard_domain x west_east\n\
dimspec i 1 standard_domain x west_east\n";
        let error = RegistryParser::parse("Registry.bad", source).unwrap_err();

        assert_eq!(error.location().line(), 2);
        assert!(matches!(
            error.kind(),
            RegistryParseErrorKind::DuplicateDimension { name } if name == "i"
        ));
    }

    #[test]
    fn rejects_unbalanced_quotes_at_the_opening_column() {
        let source = "rconfig integer days namelist,time_control 1 0 - DAYS \"open text days\n";
        let error = RegistryParser::parse("Registry.bad", source).unwrap_err();

        assert_eq!(error.location().line(), 1);
        assert_eq!(error.location().column(), 55);
        assert_eq!(error.kind(), &RegistryParseErrorKind::UnbalancedQuote);
    }

    #[test]
    fn rejects_malformed_constant_ranges() {
        let source = "dimspec z - constant=-3:3 c -\n";
        let error = RegistryParser::parse("Registry.bad", source).unwrap_err();

        assert!(matches!(
            error.kind(),
            RegistryParseErrorKind::InvalidDimensionLength { value }
                if value == "constant=-3:3"
        ));
    }

    #[test]
    fn rejects_unsupported_entry_categories_explicitly() {
        let error = RegistryParser::parse("Registry.bad", "package dyn_em - - -\n").unwrap_err();

        assert_eq!(
            error.kind(),
            &RegistryParseErrorKind::UnsupportedEntry {
                entry_kind: "package".to_owned()
            }
        );
    }

    #[test]
    fn parses_every_supported_state_staggering_flag() {
        let source = "dimspec i 1 standard_domain x west_east\n\
state real flags i dyn_em 1 xyzvmfn - FLAGS flags 1\n";
        let document = RegistryParser::parse("Registry.flags", source).unwrap();
        let staggering = document.state_variables().next().unwrap().staggering();

        assert!(staggering.is_x_staggered());
        assert!(staggering.is_y_staggered());
        assert!(staggering.is_z_staggered());
        assert!(staggering.uses_nmm_vertical_grid());
        assert!(staggering.is_microphysics_variable());
        assert!(staggering.has_full_feedback());
        assert!(staggering.has_no_feedback());
    }

    #[test]
    fn applies_wrf_default_time_level_and_empty_metadata() {
        let source = "state real cfn - misc - - irh \"cfn\" \"description\" \"\"\n";
        let document = RegistryParser::parse("Registry.defaults", source).unwrap();
        let state = document.state_variables().next().unwrap();

        assert_eq!(state.time_levels(), NonZeroU8::MIN);
        assert_eq!(state.units(), Some(""));
    }
}
