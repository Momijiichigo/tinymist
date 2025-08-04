//! Http registry for tinymist.

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, OnceLock};

#[cfg(not(target_arch = "wasm32"))]
use parking_lot::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::Response;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Certificate;
#[cfg(not(target_arch = "wasm32"))]
use tinymist_std::ImmutPath;
#[cfg(not(target_arch = "wasm32"))]
use typst::diag::{eco_format, EcoString, PackageResult, StrResult};
#[cfg(not(target_arch = "wasm32"))]
use typst::syntax::package::{PackageVersion, VersionlessPackageSpec};

#[cfg(not(target_arch = "wasm32"))]
use super::{
    DummyNotifier, Notifier, PackageError, PackageRegistry, PackageSpec, DEFAULT_REGISTRY,
};

// WASM stubs
#[cfg(target_arch = "wasm32")]
use super::{DummyNotifier, Notifier, PackageError, PackageRegistry, PackageSpec};
#[cfg(target_arch = "wasm32")]
use std::path::Path;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use typst::diag::EcoString;
#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use tinymist_std::ImmutPath;

/// The http package registry for typst.ts.
#[cfg(not(target_arch = "wasm32"))]
pub struct HttpRegistry {
    /// The path at which local packages (`@local` packages) are stored.
    package_path: Option<ImmutPath>,
    /// The path at which non-local packages (`@preview` packages) should be
    /// stored when downloaded.
    package_cache_path: Option<ImmutPath>,
    /// lazily initialized package storage.
    storage: OnceLock<PackageStorage>,
    /// The path to the certificate file to use for HTTPS requests.
    cert_path: Option<ImmutPath>,
    /// The notifier to use for progress updates.
    notifier: Arc<Mutex<dyn Notifier + Send>>,
    // package_dir_cache: RwLock<HashMap<PackageSpec, Result<ImmutPath, PackageError>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for HttpRegistry {
    fn default() -> Self {
        Self {
            notifier: Arc::new(Mutex::<DummyNotifier>::default()),
            cert_path: None,
            package_path: None,
            package_cache_path: None,

            storage: OnceLock::new(),
            // package_dir_cache: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::Deref for HttpRegistry {
    type Target = PackageStorage;

    fn deref(&self) -> &Self::Target {
        self.storage()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpRegistry {
    /// Create a new registry.
    pub fn new(
        cert_path: Option<ImmutPath>,
        package_path: Option<ImmutPath>,
        package_cache_path: Option<ImmutPath>,
    ) -> Self {
        Self {
            cert_path,
            package_path,
            package_cache_path,
            ..Default::default()
        }
    }

    /// Get `typst-kit` implementing package storage
    pub fn storage(&self) -> &PackageStorage {
        self.storage.get_or_init(|| {
            PackageStorage::new(
                self.package_cache_path
                    .clone()
                    .or_else(|| Some(dirs::cache_dir()?.join(DEFAULT_PACKAGES_SUBDIR).into())),
                self.package_path
                    .clone()
                    .or_else(|| Some(dirs::data_dir()?.join(DEFAULT_PACKAGES_SUBDIR).into())),
                self.cert_path.clone(),
                self.notifier.clone(),
            )
        })
    }

    /// Get local path option
    pub fn local_path(&self) -> Option<ImmutPath> {
        self.storage().package_path().cloned()
    }

    /// Get data & cache dir
    pub fn paths(&self) -> Vec<ImmutPath> {
        let data_dir = self.storage().package_path().cloned();
        let cache_dir = self.storage().package_cache_path().cloned();
        data_dir.into_iter().chain(cache_dir).collect::<Vec<_>>()
    }

    /// Set list of packages for testing.
    pub fn test_package_list(&self, f: impl FnOnce() -> Vec<(PackageSpec, Option<EcoString>)>) {
        self.storage().index.get_or_init(f);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PackageRegistry for HttpRegistry {
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
#[cfg(not(target_arch = "wasm32"))]
pub struct PackageStorage {
    /// The path at which non-local packages should be stored when downloaded.
    package_cache_path: Option<ImmutPath>,
    /// The path at which local packages are stored.
    package_path: Option<ImmutPath>,
    /// The downloader used for fetching the index and packages.
    cert_path: Option<ImmutPath>,
    /// The cached index of the preview namespace.
    index: OnceLock<Vec<(PackageSpec, Option<EcoString>)>>,
    notifier: Arc<Mutex<dyn Notifier + Send>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PackageStorage {
    /// Creates a new package storage for the given package paths.
    /// It doesn't fallback directories, thus you can disable the related
    /// storage by passing `None`.
    pub fn new(
        package_cache_path: Option<ImmutPath>,
        package_path: Option<ImmutPath>,
        cert_path: Option<ImmutPath>,
        notifier: Arc<Mutex<dyn Notifier + Send>>,
    ) -> Self {
        Self {
            package_cache_path,
            package_path,
            cert_path,
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
            // For other namespaces, search locally. We only search in the data
            // directory and not the cache directory, because the latter is not
            // intended for storage of local packages.
            let subdir = format!("{}/{}", spec.namespace, spec.name);
            self.package_path
                .iter()
                .flat_map(|dir| std::fs::read_dir(dir.join(&subdir)).ok())
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter_map(|path| path.file_name()?.to_string_lossy().parse().ok())
                .max()
                .ok_or_else(|| eco_format!("please specify the desired version"))
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

            threaded_http(&url, self.cert_path.as_deref(), |resp| {
                let reader = match resp.and_then(|r| r.error_for_status()) {
                    Ok(response) => response,
                    Err(err) => {
                        // todo: silent error
                        log::error!("Failed to fetch package index: {err} from {url}");
                        return vec![];
                    }
                };

                #[derive(serde::Deserialize)]
                struct RemotePackageIndex {
                    name: EcoString,
                    version: PackageVersion,
                    description: Option<EcoString>,
                }

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
            .unwrap_or_default()
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
        threaded_http(&url, self.cert_path.as_deref(), |resp| {
            let reader = match resp.and_then(|r| r.error_for_status()) {
                Ok(response) => response,
                Err(err) if matches!(err.status().map(|s| s.as_u16()), Some(404)) => {
                    return Err(PackageError::NotFound(spec.clone()))
                }
                Err(err) => return Err(PackageError::NetworkFailed(Some(eco_format!("{err}")))),
            };

            let decompressed = flate2::read::GzDecoder::new(reader);
            tar::Archive::new(decompressed)
                .unpack(package_dir)
                .map_err(|err| {
                    std::fs::remove_dir_all(package_dir).ok();
                    PackageError::MalformedArchive(Some(eco_format!("{err}")))
                })
        })
        .ok_or_else(|| PackageError::Other(Some(eco_format!("cannot spawn http thread"))))?
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn threaded_http<T: Send + Sync>(
    url: &str,
    cert_path: Option<&Path>,
    f: impl FnOnce(Result<Response, reqwest::Error>) -> T + Send + Sync,
) -> Option<T> {
    std::thread::scope(|s| {
        s.spawn(move || {
            let client_builder = reqwest::blocking::Client::builder();

            let client = if let Some(cert_path) = cert_path {
                let cert = std::fs::read(cert_path)
                    .ok()
                    .and_then(|buf| Certificate::from_pem(&buf).ok());
                if let Some(cert) = cert {
                    client_builder.add_root_certificate(cert).build().unwrap()
                } else {
                    client_builder.build().unwrap()
                }
            } else {
                client_builder.build().unwrap()
            };

            f(client.get(url).send())
        })
        .join()
        .ok()
    })
}

// WASM stub implementation
#[cfg(target_arch = "wasm32")]
pub struct HttpRegistry;

#[cfg(target_arch = "wasm32")]
impl Default for HttpRegistry {
    fn default() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
impl HttpRegistry {
    pub fn new(
        _cert_path: Option<ImmutPath>,
        _package_path: Option<ImmutPath>, 
        _package_cache_path: Option<ImmutPath>,
    ) -> Self {
        Self
    }

    pub fn set_notifier(&mut self, _notifier: Arc<parking_lot::Mutex<dyn Notifier + Send>>) {}

    pub fn package_path(&self) -> Option<&ImmutPath> {
        None
    }

    pub fn package_cache_path(&self) -> Option<&ImmutPath> {
        None
    }
}

#[cfg(target_arch = "wasm32")]
impl PackageRegistry for HttpRegistry {
    fn resolve(&self, _spec: &PackageSpec) -> Result<Arc<Path>, PackageError> {
        Err(PackageError::Other(Some(EcoString::from(
            "HTTP package registry is not supported in WASM environment"
        ))))
    }

    fn packages(&self) -> &[(PackageSpec, Option<EcoString>)] {
        &[]
    }
}

/// WASM stub for threaded_http function
#[cfg(target_arch = "wasm32")]
pub(crate) fn threaded_http<T: Send + Sync>(
    _url: &str,
    _cert_path: Option<&Path>,
    _f: impl FnOnce(Result<(), String>) -> T + Send + Sync,
) -> Option<T> {
    None
}
