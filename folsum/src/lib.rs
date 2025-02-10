#![warn(clippy::all, rust_2018_idioms)]

mod common;

mod export_csv;
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub use export_csv::export_csv;

mod gui;
pub use gui::FolsumGui;

mod logging;
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub use logging::setup_native_logging;

mod summarize;
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub use summarize::summarize_directory;
#[cfg(target_family = "wasm")]
pub use summarize::wasm_demo_summarize_directory;

mod utils;
pub use utils::sort_counts;
