#![warn(clippy::all, rust_2018_idioms)]

mod common;

mod export_csv;
#[cfg(not(target_arch = "wasm32"))]
pub use export_csv::export_csv;

mod gui;
pub use gui::FolsumGui;

mod logging;
pub use logging::setup_native_logging;

mod summarize;
#[cfg(not(target_arch = "wasm32"))]
pub use summarize::summarize_directory;
#[cfg(target_arch = "wasm32")]
pub use summarize::wasm_demo_summarize_directory;

mod utils;
pub use utils::sort_counts;
