use std::sync::Arc;

use crate::{RegistryParseError, RegistryParseErrorKind, RegistryResult, SourceLocation};

pub(super) struct RegistryTokenizer;

impl RegistryTokenizer {
    pub(super) fn tokenize(
        source_name: &Arc<str>,
        line_number: usize,
        line: &str,
    ) -> RegistryResult<Vec<String>> {
        let mut tokens = Vec::new();
        let mut token = String::new();
        let mut token_started = false;
        let mut in_quote = false;
        let mut quote_column = 1;

        for (byte_index, character) in line.char_indices() {
            if character == '"' {
                if !in_quote {
                    quote_column = byte_index + 1;
                }
                token_started = true;
                in_quote = !in_quote;
                continue;
            }

            if character == '#' {
                if in_quote {
                    token.push(' ');
                    continue;
                }
                break;
            }

            if character.is_ascii_whitespace() && !in_quote {
                if token_started {
                    tokens.push(std::mem::take(&mut token));
                    token_started = false;
                }
                continue;
            }

            if in_quote {
                token.push(character);
            } else {
                token.extend(character.to_lowercase());
            }
            token_started = true;
        }

        if in_quote {
            return Err(RegistryParseError::new(
                SourceLocation::new(source_name, line_number, quote_column),
                RegistryParseErrorKind::UnbalancedQuote,
            ));
        }

        if token_started {
            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_wrf_case_quote_and_comment_semantics() {
        let source_name: Arc<str> = Arc::from("fixture");
        let tokens = RegistryTokenizer::tokenize(
            &source_name,
            1,
            "STATE REAL T - - 1 - - T \"Case # Value\" \"K\" # comment",
        )
        .unwrap();

        assert_eq!(tokens[0], "state");
        assert_eq!(tokens[2], "t");
        assert_eq!(tokens[8], "t");
        assert_eq!(tokens[9], "Case   Value");
        assert_eq!(tokens[10], "K");
    }

    #[test]
    fn retains_empty_quoted_positional_fields() {
        let source_name: Arc<str> = Arc::from("fixture");
        let tokens = RegistryTokenizer::tokenize(
            &source_name,
            1,
            "state real cfn - misc 1 - irh \"cfn\" \"description\" \"\"",
        )
        .unwrap();

        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[10], "");
    }
}
