//! World implementation of typst for tinymist.

pub use tinymist_world as base;
pub use tinymist_world::args::*;
pub use tinymist_world::config::CompileFontOpts;
pub use tinymist_world::entry::*;
#[cfg(not(target_arch = "wasm32"))]
pub use tinymist_world::system;
pub use tinymist_world::{font, package, vfs};
pub use tinymist_world::{
    with_main, CompilerUniverse, CompilerWorld, DiagnosticFormat, EntryOpts, EntryState,
    RevisingUniverse, SourceWorld, TaskInputs,
};
