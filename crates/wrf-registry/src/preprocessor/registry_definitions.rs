use std::collections::BTreeSet;

/// Symbols visible to Registry `ifdef`/`ifndef` directives.
///
/// WRF's `registry` program fills this table from `-D` command-line flags and
/// from `define` directives encountered while preprocessing. Symbols are
/// case-sensitive and compared as whole strings, so the production form
/// `-DEM_CORE=1` defines the literal symbol `EM_CORE=1`, which is what an
/// `ifdef EM_CORE=1` line tests.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegistryDefinitions {
    symbols: BTreeSet<String>,
}

impl RegistryDefinitions {
    /// Creates an empty symbol table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a table from whole symbol strings such as `EM_CORE=1`.
    #[must_use]
    pub fn from_symbols<S: Into<String>>(symbols: impl IntoIterator<Item = S>) -> Self {
        Self {
            symbols: symbols.into_iter().map(Into::into).collect(),
        }
    }

    /// Defines one symbol; redefining an existing symbol is a no-op.
    pub fn define(&mut self, symbol: impl Into<String>) {
        self.symbols.insert(symbol.into());
    }

    /// Returns whether the exact symbol string has been defined.
    #[must_use]
    pub fn is_defined(&self, symbol: &str) -> bool {
        self.symbols.contains(symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_whole_symbol_strings_case_sensitively() {
        let definitions = RegistryDefinitions::from_symbols(["EM_CORE=1"]);

        assert!(definitions.is_defined("EM_CORE=1"));
        assert!(!definitions.is_defined("EM_CORE"));
        assert!(!definitions.is_defined("em_core=1"));
    }

    #[test]
    fn define_adds_symbols_idempotently() {
        let mut definitions = RegistryDefinitions::new();
        definitions.define("VAR4D");
        definitions.define("VAR4D");

        assert!(definitions.is_defined("VAR4D"));
    }
}
