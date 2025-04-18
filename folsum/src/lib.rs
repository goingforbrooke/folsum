#![warn(clippy::all, rust_2018_idioms)]

mod common;
pub use common::{CSV_HEADERS, DirectoryAuditStatus, FILEDATE_PREFIX_FORMAT, FileIntegrity, FileIntegrityDetail, FOLSUM_CSV_EXTENSION, FoundFile, InventoryStatus, ManifestCreationStatus};

mod export_csv;
pub use export_csv::{create_export_path, export_inventory};

mod gui;
pub use gui::FolsumGui;

mod hashers;
pub use hashers::get_md5_hash;
mod logging;
pub use logging::setup_native_logging;

mod inventory;
pub use inventory::inventory_directory;
// Summarization benchmarks.
#[cfg(feature = "bench")]
pub use inventory::tests::{generate_fake_file_paths, perform_fake_inventory};

mod audit;
pub use audit::{audit_directory_inventory, VerificationManifest};