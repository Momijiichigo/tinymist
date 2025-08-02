//! Dummy package registry implementation for testing purposes.

use std::{path::Path, sync::Arc};

use super::{PackageRegistry, PackageSpec};
use typst::diag::{PackageError, PackageResult};

/// Dummy package registry that always returns a `NotFound` error.
#[derive(Default, Debug)]
pub struct DummyRegistry;

impl PackageRegistry for DummyRegistry {
    fn resolve(&self, spec: &PackageSpec) -> PackageResult<Arc<Path>> {
        Err(PackageError::NotFound(spec.clone()))
    }
}
