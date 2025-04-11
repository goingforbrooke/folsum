// Std crates for macOS and Windows builds.
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
// Std crates for macOS and Windows builds.
use std::time::SystemTime;
use std::thread;

// Internal crates for macOS and Windows builds.
use crate::{FOLSUM_CSV_EXTENSION, ManifestCreationStatus};

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// External crates for macOS and Windows builds.
use chrono::{DateTime, Local};

// Internal crates macOS and Windows builds.
use crate::{CSV_HEADERS, FILEDATE_PREFIX_FORMAT, FoundFile};


/// Export the current inventory (show in the GUI table) to a FolSum CSV file.
///
/// # Parameters
/// - `export_file`: Path to the file that will be created.
/// - `file_paths`: Summarized files (from the GUI table).
pub fn export_inventory(
    file_paths: &Arc<Mutex<Vec<FoundFile>>>,
    manifest_creation_status: &Arc<Mutex<ManifestCreationStatus>>,
    inventory_path: &Arc<Mutex<Option<PathBuf>>>,
) -> Result<(), &'static str> {
    // Copy Arcs so we can access them in a separate thread that's dedicated to this CSV dump.
    let file_paths_copy: Arc<Mutex<Vec<FoundFile>>> = file_paths.clone();
    let manifest_creation_status: Arc<Mutex<ManifestCreationStatus>> = manifest_creation_status.clone();
    let inventory_path = inventory_path.clone();

    thread::spawn(move || {
        // Note that the creation of a verification manifest export file has begun.
        *manifest_creation_status.lock().unwrap() = ManifestCreationStatus::InProgress;

        // Make a place to put file paths that'll be written to the CSV file and include column headers.
        let mut csv_rows = CSV_HEADERS.to_string();
        let locked_file_paths: MutexGuard<'_, Vec<FoundFile>> = file_paths_copy.lock().unwrap();
        for found_file in locked_file_paths.iter() {
            let show_path = found_file.file_path.to_string_lossy();
            let file_md5 = &found_file.md5_hash;
            // Ensure that there are no commas or newlines in this extension's name that would disrupt the output format.
            // todo: Replace problematic CSV characters with a placeholder instead of erroring out.
            assert!(!show_path.contains('\n') && !show_path.contains(','));
            let csv_row = format!("{show_path},{file_md5}\n");
            csv_rows.push_str(&csv_row)
        }
        let export_path = create_export_path(&inventory_path);
        // Create a CSV file to write the extension types and their counts to, overwriting it if it already exists.
        let mut csv_export = File::create(&export_path).expect("Failed to create CSV export file");
        // Write the CSV's content to the export file.
        csv_export.write_all(csv_rows.as_bytes()).expect("Failed to write contents to CSV export file");

        info!("Exported inventory to: {export_path:?}");
        // Note that the creation of a verification manifest export file has completed.
        // This will be reset to "not started" when "Audit" is clicked.
        *manifest_creation_status.lock().unwrap() = ManifestCreationStatus::Done(export_path.clone());
    });
    Ok(())
}

/// Create a path for a new export file.
///
/// New export files *should* be created inside the directory that they inventoried.
pub fn create_export_path(inventory_path: &Arc<Mutex<Option<PathBuf>>>) -> PathBuf {
    let locked_inventory_path = inventory_path.lock().unwrap();
    let inventory_path_copy = locked_inventory_path.clone();
    drop(locked_inventory_path);

    let date_today: DateTime<Local> = DateTime::from(SystemTime::now());
    // Prefix the export filename with the non-zero padded date and time.
    let formatted_date = date_today.format(FILEDATE_PREFIX_FORMAT).to_string();

    // Extract the name of the inventoried directory so we can use it to name the export file.
    // Assume that an inventory directory's been selected b/c we checked in the export prerequisites before this.
    let inventory_path_copy = inventory_path_copy.unwrap();
    let raw_directory_name = inventory_path_copy.file_name().unwrap();
    let display_directory_name = raw_directory_name.to_string_lossy().to_string();

    // Name the export file `YYYY-MM-DD-HH-MM_<inventoried_folder_name>.folsum.csv`. (we'll add the .csv later).
    let export_filename = format!("{formatted_date}_{display_directory_name}{FOLSUM_CSV_EXTENSION}");
    // Put the export file into the directory that was assessed.
    let export_path: PathBuf = [inventory_path_copy, PathBuf::from(export_filename)].iter().collect();

    debug!("Created path for new export file: {export_path:?}");
    export_path
}
