use wrf_registry::StateVariable;

use crate::{WrfIoError, WrfIoResult};

/// Decides restart-stream membership from a Registry I/O specification.
///
/// This mirrors the second scan in WRF's `tools/reg_parse.c`: a lowercase `r`
/// selects the restart stream, `{unit}` stream selectors are consumed as one
/// token, and the `=(package:fields)` payload after an `f`, `d`, `u`, or `s`
/// nesting specifier is skipped so letters inside interpolation names cannot
/// select streams. An unmatched `{` is a hard Registry error upstream and a
/// typed error here; an unterminated `=(...)` payload consumes the remainder
/// of the specification exactly as upstream does.
pub(crate) struct RestartStreamSelection;

impl RestartStreamSelection {
    /// Reports whether the state is written to WRF's restart stream.
    pub(crate) fn is_selected(state: &StateVariable) -> WrfIoResult<bool> {
        let Some(specification) = state.io_specification() else {
            return Ok(false);
        };

        let characters: Vec<char> = specification.chars().collect();
        let mut selected = false;
        let mut index = 0;
        while index < characters.len() {
            match characters[index].to_ascii_lowercase() {
                '{' => {
                    let closing = characters[index..]
                        .iter()
                        .position(|&character| character == '}')
                        .ok_or_else(|| WrfIoError::InvalidIoSpecification {
                            state: state.name().to_owned(),
                            value: specification.to_owned(),
                        })?;
                    index += closing + 1;
                }
                'r' => {
                    selected = true;
                    index += 1;
                }
                'f' | 'd' | 'u' | 's' if characters.get(index + 1) == Some(&'=') => {
                    let closing = characters[index + 2..]
                        .iter()
                        .position(|&character| character == ')');
                    index = match closing {
                        Some(offset) => index + 2 + offset + 1,
                        None => characters.len(),
                    };
                }
                _ => index += 1,
            }
        }
        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use wrf_registry::RegistryParser;

    use super::*;

    fn state_with_io(io_specification: &str) -> StateVariable {
        let source = format!(
            "dimspec i 1 standard_domain x west_east\n\
             state real field i dyn_em 1 - {io_specification} \"FIELD\" \"d\" \"u\"\n"
        );
        let document = RegistryParser::parse("fixture", &source).unwrap();
        document.state_variables().next().unwrap().clone()
    }

    #[test]
    fn is_selected_finds_restart_letter_and_skips_interpolation_payloads() {
        assert!(RestartStreamSelection::is_selected(&state_with_io("irh")).unwrap());
        assert!(
            RestartStreamSelection::is_selected(&state_with_io("i0rhusdf=(bdy_interp:dt)"))
                .unwrap()
        );
        assert!(
            !RestartStreamSelection::is_selected(&state_with_io("i0husdf=(bdy_interp:dt)"))
                .unwrap()
        );
        assert!(!RestartStreamSelection::is_selected(&state_with_io("h")).unwrap());
    }

    #[test]
    fn is_selected_treats_braced_units_as_opaque_and_rejects_unmatched_braces() {
        assert!(!RestartStreamSelection::is_selected(&state_with_io("i{3r}h")).unwrap());
        assert!(matches!(
            RestartStreamSelection::is_selected(&state_with_io("i{3rh")),
            Err(WrfIoError::InvalidIoSpecification { .. })
        ));
    }

    #[test]
    fn is_selected_returns_false_without_a_specification() {
        assert!(!RestartStreamSelection::is_selected(&state_with_io("-")).unwrap());
    }
}
