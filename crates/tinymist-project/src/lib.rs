//! Project Model for tinymist

mod args;
mod compiler;
mod entry;
mod model;

#[cfg(feature = "lsp")]
mod lock;
#[cfg(all(feature = "lsp", not(target_arch = "wasm32")))]
mod lsp;
#[cfg(all(feature = "system", not(target_arch = "wasm32")))]
mod watch;
#[cfg(all(feature = "system", not(target_arch = "wasm32")))]
pub mod world;

pub use args::*;
pub use compiler::*;
pub use entry::*;
pub use model::*;

#[cfg(all(feature = "lsp", not(target_arch = "wasm32")))]
pub use lock::*;
#[cfg(all(feature = "lsp", not(target_arch = "wasm32")))]
pub use lsp::*;
#[cfg(all(feature = "system", not(target_arch = "wasm32")))]
pub use watch::*;
#[cfg(all(feature = "system", not(target_arch = "wasm32")))]
pub use world::*;

pub use tinymist_world::{CompileSignal, CompileSnapshot, ProjectInsId};

/// The default project route priority assigned to user actions.
pub const PROJECT_ROUTE_USER_ACTION_PRIORITY: u32 = 256;
