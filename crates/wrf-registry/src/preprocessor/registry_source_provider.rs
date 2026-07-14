use std::path::Path;

/// Capability to read Registry source text for the preprocessor.
///
/// Returning `None` means the path cannot be opened; the preprocessor treats
/// every open failure the same way, matching the `fopen` checks in WRF's
/// `pre_parse`. Implementations must not mutate any state observed by the
/// preprocessor between calls with equal paths.
pub trait RegistrySourceProvider {
    /// Returns the complete source at `path`, or `None` when it is unreadable.
    fn read_source(&self, path: &Path) -> Option<String>;
}
