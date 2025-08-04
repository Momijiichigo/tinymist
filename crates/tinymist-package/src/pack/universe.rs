use super::*;
use crate::registry::DEFAULT_REGISTRY;

/// A package in the universe registry.
#[derive(Debug, Clone)]
pub struct UniversePack {
    /// The package specifier.
    pub specifier: PackageSpec,
}

impl UniversePack {
    /// Creates a new `UniversePack` instance.
    pub fn new(specifier: PackageSpec) -> Self {
        Self { specifier }
    }
}

impl PackFs for UniversePack {
    fn read_all(
        &mut self,
        f: &mut (dyn FnMut(&str, PackFile) -> PackageResult<()> + Send + Sync),
    ) -> PackageResult<()> {
        let spec = &self.specifier;
        assert_eq!(spec.namespace, "preview");

        #[cfg(not(target_arch = "wasm32"))]
        {
            let url = format!(
                "{DEFAULT_REGISTRY}/preview/{}-{}.tar.gz",
                spec.name, spec.version
            );

            HttpPack::new(self.specifier.clone(), url).read_all(f)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(PackageError::Other(Some("Universe packages not supported in WASM".into())))
        }
    }
}

impl Pack for UniversePack {}
impl PackExt for UniversePack {}
