#![warn(clippy::all, rust_2018_idioms)]

mod common;
pub use common::{CSV_HEADERS, DirectoryVerificationStatus, FILEDATE_PREFIX_FORMAT, FileIntegrity, FOLSUM_CSV_EXTENSION, FoundFile, IntegrityDetail, ManifestCreationStatus, SummarizationStatus};

mod export_csv;
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub use export_csv::{create_export_path, export_csv};

mod gui;
pub use gui::FolsumGui;

mod hashers;
pub use hashers::get_md5_hash;
 mod logging; #[cfg(any(target_family = "unix", target_family = "windows"))]
pub use logging::setup_native_logging;

mod summarize;
// Summarization items for native builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub use summarize::summarize_directory;
// Summarization items for WASM builds.
#[cfg(target_family = "wasm")]
pub use summarize::wasm_demo_summarize_directory;
// Summarization benchmarks for native builds.
#[cfg(feature = "bench")]
pub use summarize::tests::run_fake_summarization;

mod verification;
pub use verification::{find_verification_manifest_files, find_previous_manifest, verify_summarization, VerificationManifest};