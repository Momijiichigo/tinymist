use std::{io::Cursor, path::Path};

use typst::diag::{PackageError, PackageResult};

pub(crate) fn get(url: &str) -> PackageResult<Vec<u8>> {
    let mut res = ehttp::get(url).map_err(|e| PackageError::NetworkFailed(Some(e.into())))?;
    if res.status != 200 {
        return Err(PackageError::NetworkFailed(Some(
            format!("status code {}", res.status).into(),
        )));
    }

    Ok(res.bytes)
}

pub(crate) fn get_to_value<T: for<'de> serde::Deserialize<'de>>(url: &str) -> PackageResult<T> {
    let mut res = ehttp::get(url).map_err(|e| PackageError::NetworkFailed(Some(e.into())))?;
    if res.status != 200 {
        return Err(PackageError::NetworkFailed(Some(
            format!("status code {}", res.status).into(),
        )));
    }

    serde_json::from_slice(&res.bytes).map_err(|e| PackageError::MalformedArchive(Some(e.into())))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn threaded_http<T: Send + Sync>(
    url: &str,
    _cert_path: Option<&Path>,
    f: impl FnOnce(PackageResult<Cursor<Vec<u8>>>) -> T + Send + Sync,
) -> Option<T> {
    std::thread::scope(|s| {
        s.spawn(move || {
            let res = get(url);
            f(res.map(Cursor::new))
        })
        .join()
        .ok()
    })
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn threaded_http<T: Send + Sync>(
    url: &str,
    _cert_path: Option<&Path>,
    f: impl FnOnce(PackageResult<Cursor<Vec<u8>>>) -> T + Send + Sync,
) -> Option<T> {
    let res = get(url);
    Some(f(res.map(Cursor::new)))
}
