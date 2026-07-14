use std::fs;
use std::path::Path;

use crate::preprocessor::registry_source_provider::RegistrySourceProvider;

/// Reads Registry sources directly from the local file system.
pub struct FileSystemSourceProvider;

impl RegistrySourceProvider for FileSystemSourceProvider {
    fn read_source(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_a_missing_path() {
        let provider = FileSystemSourceProvider;

        assert!(
            provider
                .read_source(Path::new("definitely/not/a/registry/file"))
                .is_none()
        );
    }
}
