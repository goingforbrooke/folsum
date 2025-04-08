#![warn(clippy::all, rust_2018_idioms)]

mod common;
pub use common::{CSV_HEADERS, DirectoryAuditStatus, FILEDATE_PREFIX_FORMAT, FileIntegrity, FOLSUM_CSV_EXTENSION, FoundFile, FileIntegrityDetail, ManifestCreationStatus, InventoryStatus};

mod export_csv;
pub use export_csv::{create_export_path, export_csv};

mod gui;
pub use gui::FolsumGui;

mod hashers;
pub use hashers::get_md5_hash;
mod logging;
pub use logging::setup_native_logging;

mod summarize;
pub use summarize::summarize_directory;
// Summarization benchmarks.
#[cfg(feature = "bench")]
pub use summarize::tests::run_fake_summarization;

mod verification;
pub use verification::{audit_summarization, VerificationManifest};