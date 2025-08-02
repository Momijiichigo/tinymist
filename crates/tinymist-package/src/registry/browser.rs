#![cfg(target_arch = "wasm32")]

//! Browser registry for tinymist.
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;
use tinymist_std::ImmutPath;
use typst::diag::{eco_format, EcoString, PackageResult, StrResult};
use typst::syntax::package::{PackageVersion, VersionlessPackageSpec};
use wasm_bindgen::JsValue;

use super::{
    DummyNotifier, Notifier, PackageError, PackageRegistry, PackageSpec, DEFAULT_REGISTRY,
};

type BrowserResult<T> = Result<T, JsValue>;

fn to_package_error(err: JsValue) -> PackageError {
    PackageError::NetworkFailed(Some(eco_format!("{:?}", err)))
}

async fn fetch_bytes(url: &str) -> BrowserResult<Vec<u8>> {
    let response = reqwest::get(url).await?;
    let response = response.error_for_status()?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

/// The http package registry for typst.ts.
pub struct BrowserRegistry {
    /// The path at which local packages (`@local` packages) are stored.
    package_path: Option<ImmutPath>,
    /// The path at which non-local packages (`@preview` packages) should be
    /// stored when downloaded.
    package_cache_path: Option<ImmutPath>,
    /// lazily initialized package storage.
    storage: OnceLock<PackageStorage>,
    /// The notifier to use for progress updates.
    notifier: Arc<Mutex<dyn Notifier + Send>>,
}

impl Default for BrowserRegistry {
    fn default() -> Self {
        Self {
            notifier: Arc::new(Mutex::<DummyNotifier>::default()),
            package_path: None,
            package_cache_path: None,
            storage: OnceLock::new(),
        }
    }
}

impl std::ops::Deref for BrowserRegistry {
    type Target = PackageStorage;

    fn deref(&self) -> &Self::Target {
        self.storage()
    }
}

impl BrowserRegistry {
    /// Create a new registry.
    pub fn new(
        package_path: Option<ImmutPath>,
        package_cache_path: Option<ImmutPath>,
    ) -> Self {
        Self {
            package_path,
            package_cache_path,
            ..Default::default()
        }
    }

    /// Get `typst-kit` implementing package storage
    pub fn storage(&self) -> &PackageStorage {
        self.storage.get_or_init(|| {
            PackageStorage::new(
                self.package_cache_path.clone(),
                self.package_path.clone(),
                self.notifier.clone(),
            )
        })
    }

    /// Get local path option
    pub fn local_path(&self) -> Option<ImmutPath> {
        self.storage().package_path().cloned()
    }
}

impl PackageRegistry for BrowserRegistry {
    fn resolve(&self, spec: &PackageSpec) -> Result<ImmutPath, PackageError> {
        self.storage().prepare_package(spec)
    }

    fn packages(&self) -> &[(PackageSpec, Option<EcoString>)] {
        self.storage().download_index()
    }
}

/// The default packages sub directory within the package and package cache
/// paths.
pub const DEFAULT_PACKAGES_SUBDIR: &str = "typst/packages";

/// Holds information about where packages should be stored and downloads them
/// on demand, if possible.
pub struct PackageStorage {
    /// The path at which non-local packages should be stored when downloaded.
    package_cache_path: Option<ImmutPath>,
    /// The path at which local packages are stored.
    package_path: Option<ImmutPath>,
    /// The cached index of the preview namespace.
    index: OnceLock<Vec<(PackageSpec, Option<EcoString>)>>,
    notifier: Arc<Mutex<dyn Notifier + Send>>,
}

impl PackageStorage {
    /// Creates a new package storage for the given package paths.
    /// It doesn't fallback directories, thus you can disable the related
    /// storage by passing `None`.
    pub fn new(
        package_cache_path: Option<ImmutPath>,
        package_path: Option<ImmutPath>,
        notifier: Arc<Mutex<dyn Notifier + Send>>,
    ) -> Self {
        Self {
            package_cache_path,
            package_path,
            notifier,
            index: OnceLock::new(),
        }
    }

    /// Returns the path at which non-local packages should be stored when
    /// downloaded.
    pub fn package_cache_path(&self) -> Option<&ImmutPath> {
        self.package_cache_path.as_ref()
    }

    /// Returns the path at which local packages are stored.
    pub fn package_path(&self) -> Option<&ImmutPath> {
        self.package_path.as_ref()
    }

    /// Make a package available in the on-disk cache.
    pub fn prepare_package(&self, spec: &PackageSpec) -> PackageResult<ImmutPath> {
        let subdir = format!("{}/{}/{}", spec.namespace, spec.name, spec.version);

        if let Some(packages_dir) = &self.package_path {
            let dir = packages_dir.join(&subdir);
            if dir.exists() {
                return Ok(dir.into());
            }
        }

        if let Some(cache_dir) = &self.package_cache_path {
            let dir = cache_dir.join(&subdir);
            if dir.exists() {
                return Ok(dir.into());
            }

            // Download from network if it doesn't exist yet.
            if spec.namespace == "preview" {
                self.download_package(spec, &dir)?;
                if dir.exists() {
                    return Ok(dir.into());
                }
            }
        }

        Err(PackageError::NotFound(spec.clone()))
    }

    /// Try to determine the latest version of a package.
    pub fn determine_latest_version(
        &self,
        spec: &VersionlessPackageSpec,
    ) -> StrResult<PackageVersion> {
        if spec.namespace == "preview" {
            // For `@preview`, download the package index and find the latest
            // version.
            self.download_index()
                .iter()
                .filter(|(package, _)| package.name == spec.name)
                .map(|(package, _)| package.version)
                .max()
                .ok_or_else(|| eco_format!("failed to find package {spec}"))
        } else {
            // For other namespaces, we do not have a way to search for them yet.
            Err(eco_format!(
                "please specify the desired version for package {spec}"
            ))
        }
    }

    /// Get the cached package index without network access.
    pub fn cached_index(&self) -> Option<&[(PackageSpec, Option<EcoString>)]> {
        self.index.get().map(Vec::as_slice)
    }

    /// Download the package index. The result of this is cached for efficiency.
    pub fn download_index(&self) -> &[(PackageSpec, Option<EcoString>)] {
        self.index.get_or_init(|| {
            let url = format!("{DEFAULT_REGISTRY}/preview/index.json");

            let fut = fetch_bytes(&url);

            let bytes = match pollster::block_on(fut) {
                Ok(bytes) => bytes,
                Err(err) => {
                    log::error!("Failed to fetch package index: {err:?} from {url}");
                    return vec![];
                }
            };

            #[derive(serde::Deserialize)]
            struct RemotePackageIndex {
                name: EcoString,
                version: PackageVersion,
                description: Option<EcoString>,
            }

            let reader = Cursor::new(bytes);
            let indices: Vec<RemotePackageIndex> = match serde_json::from_reader(reader) {
                Ok(index) => index,
                Err(err) => {
                    log::error!("Failed to parse package index: {err} from {url}");
                    return vec![];
                }
            };

            indices
                .into_iter()
                .map(|index| {
                    (
                        PackageSpec {
                            namespace: "preview".into(),
                            name: index.name,
                            version: index.version,
                        },
                        index.description,
                    )
                })
                .collect::<Vec<_>>()
        })
    }

    /// Download a package over the network.
    ///
    /// # Panics
    /// Panics if the package spec namespace isn't `preview`.
    pub fn download_package(&self, spec: &PackageSpec, package_dir: &Path) -> PackageResult<()> {
        assert_eq!(spec.namespace, "preview");

        let url = format!(
            "{DEFAULT_REGISTRY}/preview/{}-{}.tar.gz",
            spec.name, spec.version
        );

        self.notifier.lock().downloading(spec);

        let fut = fetch_bytes(&url);

        let bytes = match pollster::block_on(fut) {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(to_package_error(err));
            }
        };

        let decompressed = flate2::read::GzDecoder::new(Cursor::new(bytes));
        tar::Archive::new(decompressed)
            .unpack(package_dir)
            .map_err(|err| {
                // todo: remove dir all in wasm
                // std::fs::remove_dir_all(package_dir).ok();
                PackageError::MalformedArchive(Some(eco_format!("{err}")))
            })
    }
}
