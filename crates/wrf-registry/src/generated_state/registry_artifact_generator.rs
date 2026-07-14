use std::fmt::Write;

use crate::{
    ConfigurationEntryCount, CoordinateAxis, DimensionLength, RegistryDocument, RegistryEntry,
    RegistryGenerationError, RegistryGenerationResult, StateVariable,
};

/// Generates the first selected set of WRF-compatible Registry artifacts.
pub struct RegistryArtifactGenerator;

impl RegistryArtifactGenerator {
    /// Generates exact WRF v4.7.1 include text for the supported Registry subset.
    ///
    /// Generation requires exactly one standard-domain dimension at each order
    /// from one through three. Parsed four-dimensional scalar-array members are
    /// rejected until their dedicated generator slice is implemented.
    pub fn generate(
        document: &RegistryDocument,
    ) -> RegistryGenerationResult<crate::GeneratedRegistryArtifacts> {
        for state in document.state_variables() {
            if state.dimensions().is_scalar_array_member() {
                return Err(RegistryGenerationError::UnsupportedScalarArrayMember {
                    state_name: state.name().to_owned(),
                });
            }
        }

        let model_data_order = Self::model_data_order(document)?;
        Ok(crate::GeneratedRegistryArtifacts {
            state_struct: Self::state_struct(document),
            namelist_defines: Self::namelist_defines(document),
            namelist_defaults: Self::namelist_defaults(document),
            namelist_statements: Self::namelist_statements(document),
            model_data_order,
            state_metadata: Self::state_metadata(document),
        })
    }

    fn state_struct(document: &RegistryDocument) -> String {
        let mut body = String::new();

        for entry in document.entries() {
            match entry {
                RegistryEntry::RuntimeConfiguration(configuration) => Self::write_declaration(
                    &mut body,
                    configuration.value_type().as_fortran(),
                    "",
                    "",
                    configuration.name(),
                ),
                RegistryEntry::State(state) if state.dimensions().names().is_empty() => {
                    Self::write_state_declarations(&mut body, state);
                }
                RegistryEntry::Dimension(_)
                | RegistryEntry::State(_)
                | RegistryEntry::Package(_) => {}
            }
        }

        for entry in document.entries() {
            if let RegistryEntry::State(state) = entry
                && !state.dimensions().names().is_empty()
            {
                Self::write_state_declarations(&mut body, state);
            }
        }

        Self::wrap_include("inc/state_struct.inc", &body)
    }

    fn write_state_declarations(output: &mut String, state: &StateVariable) {
        let dimensions = Self::colon_dimensions(state.dimensions().names().len());
        let pointer = if dimensions.is_empty() {
            ""
        } else {
            ",POINTER"
        };
        let time_levels = state.time_levels().get();
        for time_level in 1..=time_levels {
            let name = if time_levels > 1 {
                format!("{}_{time_level}", state.name())
            } else {
                state.name().to_owned()
            };
            Self::write_declaration(
                output,
                state.value_type().as_fortran(),
                &dimensions,
                pointer,
                &name,
            );
        }

        if state.dimensions().is_boundary_array() {
            let boundary_dimensions = Self::colon_dimensions(4);
            Self::write_declaration(
                output,
                state.value_type().as_fortran(),
                &boundary_dimensions,
                ",POINTER",
                &format!("{}_b", state.name()),
            );
            Self::write_declaration(
                output,
                state.value_type().as_fortran(),
                &boundary_dimensions,
                ",POINTER",
                &format!("{}_bt", state.name()),
            );
        }
    }

    fn namelist_defines(document: &RegistryDocument) -> String {
        let mut body = String::from("integer    :: first_item_in_struct\n");
        for configuration in document.runtime_configurations() {
            match configuration.entry_count() {
                ConfigurationEntryCount::Scalar => {
                    writeln!(
                        body,
                        "{} :: {}",
                        configuration.value_type().as_fortran(),
                        configuration.name()
                    )
                    .expect("writing to String cannot fail");
                }
                ConfigurationEntryCount::Expression(expression) => {
                    writeln!(
                        body,
                        "{} , DIMENSION({expression}) :: {}",
                        configuration.value_type().as_fortran(),
                        configuration.name()
                    )
                    .expect("writing to String cannot fail");
                }
            }
        }
        body.push_str("integer    :: last_item_in_struct\n");
        Self::wrap_include("inc/namelist_defines.inc", &body)
    }

    fn namelist_defaults(document: &RegistryDocument) -> String {
        let mut body = String::new();
        for configuration in document.runtime_configurations() {
            let Some(default_value) = configuration.default_value() else {
                continue;
            };
            if configuration
                .value_type()
                .as_fortran()
                .starts_with("character")
            {
                writeln!(body, "{} = \"{default_value}\"", configuration.name())
                    .expect("writing to String cannot fail");
            } else {
                writeln!(body, "{} = {default_value}", configuration.name())
                    .expect("writing to String cannot fail");
            }
        }
        Self::wrap_include("inc/namelist_defaults.inc", &body)
    }

    fn namelist_statements(document: &RegistryDocument) -> String {
        let mut body = String::new();
        for configuration in document.runtime_configurations() {
            let Some(section) = configuration.namelist_section() else {
                continue;
            };
            writeln!(body, "NAMELIST /{section}/ {}", configuration.name())
                .expect("writing to String cannot fail");
        }
        Self::wrap_include("inc/namelist_statements.inc", &body)
    }

    fn model_data_order(document: &RegistryDocument) -> RegistryGenerationResult<String> {
        let mut axes = [None; 3];
        for dimension in document
            .dimensions()
            .filter(|dimension| matches!(dimension.length(), DimensionLength::StandardDomain))
        {
            let order = dimension
                .order()
                .expect("parser validates standard dimensions");
            let slot = &mut axes[usize::from(order - 1)];
            if slot.replace(dimension.axis()).is_some() {
                return Err(RegistryGenerationError::DuplicateStandardDimensionOrder { order });
            }
        }

        let mut order_name = String::new();
        for (index, axis) in axes.into_iter().enumerate() {
            let order = u8::try_from(index + 1).expect("three dimensions fit in u8");
            let axis =
                axis.ok_or(RegistryGenerationError::MissingStandardDimensionOrder { order })?;
            order_name.push(Self::axis_letter(axis));
        }
        let body = format!("INTEGER , PARAMETER :: model_data_order   = DATA_ORDER_{order_name}\n");
        Ok(Self::wrap_include("inc/model_data_order.inc", &body))
    }

    fn state_metadata(document: &RegistryDocument) -> String {
        let dimensions: Vec<_> = document.dimensions().collect();
        let mut output = String::new();
        for state in document.state_variables() {
            let memory_order: String = state
                .dimensions()
                .names()
                .iter()
                .filter_map(|name| dimensions.iter().find(|dimension| dimension.name() == name))
                .map(|dimension| Self::axis_letter(dimension.axis()))
                .collect();
            let time_levels = state.time_levels().get();
            for time_level in 1..=time_levels {
                let suffix = (time_levels > 1).then(|| format!("_{time_level}"));
                Self::write_metadata_record(
                    &mut output,
                    state,
                    suffix.as_deref().unwrap_or(""),
                    state.description().unwrap_or("-"),
                    state.units().unwrap_or("-"),
                    &memory_order,
                    if time_levels > 1 {
                        200 + u16::from(time_level)
                    } else {
                        0
                    },
                );
            }
            if state.dimensions().is_boundary_array() {
                Self::write_metadata_record(
                    &mut output,
                    state,
                    "_b",
                    &format!("bdy {}", state.description().unwrap_or("-")),
                    state.units().unwrap_or("-"),
                    "C",
                    0,
                );
                Self::write_metadata_record(
                    &mut output,
                    state,
                    "_bt",
                    &format!("bdy tend {}", state.description().unwrap_or("-")),
                    &format!("({})/dt", state.units().unwrap_or("-")),
                    "C",
                    0,
                );
            }
        }
        output
    }

    fn write_metadata_record(
        output: &mut String,
        state: &StateVariable,
        suffix: &str,
        description: &str,
        units: &str,
        memory_order: &str,
        time_level: u16,
    ) {
        let data_name = if matches!(suffix, "_b" | "_bt") {
            state.name()
        } else {
            state.data_name().unwrap_or(state.name())
        }
        .to_ascii_uppercase();
        writeln!(
            output,
            "VarName={}{suffix}|DataName={data_name}{suffix_upper}|Description={description}|Units={units}|MemoryOrder={memory_order}|Ntl={time_level}|Ndim={}",
            state.name(),
            state.dimensions().names().len(),
            suffix_upper = suffix.to_ascii_uppercase(),
        )
        .expect("writing to String cannot fail");
    }

    fn write_declaration(
        output: &mut String,
        value_type: &str,
        dimensions: &str,
        pointer: &str,
        name: &str,
    ) {
        writeln!(
            output,
            "{value_type:<10}{dimensions:<20}{pointer:<10} :: {name}"
        )
        .expect("writing to String cannot fail");
    }

    fn colon_dimensions(count: usize) -> String {
        if count == 0 {
            return String::new();
        }
        format!(",DIMENSION({})", vec![":"; count].join(","))
    }

    fn axis_letter(axis: CoordinateAxis) -> char {
        match axis {
            CoordinateAxis::X => 'X',
            CoordinateAxis::Y => 'Y',
            CoordinateAxis::Z => 'Z',
            CoordinateAxis::Constant => 'C',
        }
    }

    fn wrap_include(path: &str, body: &str) -> String {
        format!(
            "!STARTOFREGISTRYGENERATEDINCLUDE '{path}'\n!\n! WARNING This file is generated automatically by use_registry\n! using the data base in the file named Registry.\n! Do not edit.  Your changes to this file will be lost.\n!\n{body}!ENDOFREGISTRYGENERATEDINCLUDE\n"
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{RegistryDefinitions, RegistryParser};

    use super::*;

    const STATE_STRUCT: &str = include_str!("../../../../parity/registry/golden/state_struct.inc");
    const NAMELIST_DEFINES: &str =
        include_str!("../../../../parity/registry/golden/namelist_defines.inc");
    const NAMELIST_DEFAULTS: &str =
        include_str!("../../../../parity/registry/golden/namelist_defaults.inc");
    const NAMELIST_STATEMENTS: &str =
        include_str!("../../../../parity/registry/golden/namelist_statements.inc");
    const MODEL_DATA_ORDER: &str =
        include_str!("../../../../parity/registry/golden/model_data_order.inc");
    const STATE_METADATA: &str =
        include_str!("../../../../parity/registry/golden/state_metadata.txt");

    #[test]
    fn generates_selected_arw_artifacts() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../parity/registry/fixtures/registry_arw_slice");
        let definitions = RegistryDefinitions::from_symbols(["PARITY_SLICE=1"]);
        let document = RegistryParser::parse_file(fixture, &definitions).unwrap();
        let artifacts = RegistryArtifactGenerator::generate(&document).unwrap();

        assert_eq!(artifacts.state_struct(), STATE_STRUCT);
        assert_eq!(artifacts.namelist_defines(), NAMELIST_DEFINES);
        assert_eq!(artifacts.namelist_defaults(), NAMELIST_DEFAULTS);
        assert_eq!(artifacts.namelist_statements(), NAMELIST_STATEMENTS);
        assert_eq!(artifacts.model_data_order(), MODEL_DATA_ORDER);
        assert_eq!(artifacts.state_metadata(), STATE_METADATA);
    }
}
