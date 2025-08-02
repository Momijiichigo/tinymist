use ecow::eco_format;
use typst::diag::{PackageError, PackageResult};

use super::*;
#[cfg(all(feature = "http-registry", not(target_arch = "wasm32")))]
use crate::registry::http::threaded_http;

/// A package in the remote http.
#[derive(Clone)]
pub struct HttpPack<S> {
    /// The package specifier.
    pub specifier: PackageSpec,
    /// The url of the package.
    pub url: S,
}

impl<S: AsRef<str>> HttpPack<S> {
    /// Creates a new `HttpPack` instance.
    pub fn new(specifier: PackageSpec, url: S) -> Self {
        Self { specifier, url }
    }
}

impl<S: AsRef<str>> fmt::Debug for HttpPack<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpPack({})", self.url.as_ref())
    }
}

impl<S: AsRef<str>> PackFs for HttpPack<S> {
    #[cfg(all(feature = "http-registry", not(target_arch = "wasm32")))]
    fn read_all(
        &mut self,
        f: &mut (dyn FnMut(&str, PackFile) -> PackageResult<()> + Send + Sync),
    ) -> PackageResult<()> {
        let spec = &self.specifier;
        let url = self.url.as_ref();
        threaded_http(url, None, |resp| {
            let reader = match resp.and_then(|r| r.error_for_status()) {
                Ok(response) => response,
                Err(err) if matches!(err.status().map(|s| s.as_u16()), Some(404)) => {
                    return Err(PackageError::NotFound(spec.clone()))
                }
                Err(err) => return Err(PackageError::NetworkFailed(Some(eco_format!("{err}")))),
            };

            let decompressed = flate2::read::GzDecoder::new(reader);
            let mut tarbar = TarballPack::new(decompressed);

            tarbar.read_all(f)
        })
        .ok_or_else(|| PackageError::Other(Some(eco_format!("cannot spawn http thread"))))?
    }

    #[cfg(not(all(feature = "http-registry", not(target_arch = "wasm32"))))]
    fn read_all(
        &mut self,
        _f: &mut (dyn FnMut(&str, PackFile) -> PackageResult<()> + Send + Sync),
    ) -> PackageResult<()> {
        panic!("http-registry feature is not enabled or not supported on this target")
    }
}

impl<S: AsRef<str>> Pack for HttpPack<S> {}
impl<P: AsRef<str>> PackExt for HttpPack<P> {}
