use crate::MicrophysicsScheme;

/// Scheme-specific reusable storage owned by one microphysics driver.
///
/// Disabled microphysics carries no numerical storage, preserving WRF's
/// `mp_physics == 0` early return without imposing Kessler region requirements
/// or allocating Kessler scratch. A Kessler workspace remains backend-native.
#[derive(Debug)]
pub struct MicrophysicsDriverWorkspace<KesslerWorkspace> {
    scheme: MicrophysicsScheme,
    kessler_workspace: Option<KesslerWorkspace>,
}

impl<KesslerWorkspace> MicrophysicsDriverWorkspace<KesslerWorkspace> {
    pub(crate) const fn disabled() -> Self {
        Self {
            scheme: MicrophysicsScheme::Disabled,
            kessler_workspace: None,
        }
    }

    pub(crate) const fn kessler(kessler_workspace: KesslerWorkspace) -> Self {
        Self {
            scheme: MicrophysicsScheme::Kessler,
            kessler_workspace: Some(kessler_workspace),
        }
    }

    /// Returns the scheme for which this workspace was created.
    pub const fn scheme(&self) -> MicrophysicsScheme {
        self.scheme
    }

    pub(crate) fn kessler_workspace_mut(&mut self) -> Option<&mut KesslerWorkspace> {
        self.kessler_workspace.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_workspace_carries_no_kessler_storage() {
        let mut workspace = MicrophysicsDriverWorkspace::<()>::disabled();

        assert_eq!(workspace.scheme(), MicrophysicsScheme::Disabled);
        assert_eq!(workspace.kessler_workspace_mut(), None);
    }

    #[test]
    fn kessler_workspace_preserves_backend_storage() {
        let mut workspace = MicrophysicsDriverWorkspace::kessler(7_u32);

        assert_eq!(workspace.scheme(), MicrophysicsScheme::Kessler);
        assert_eq!(workspace.kessler_workspace_mut(), Some(&mut 7_u32));
    }
}
