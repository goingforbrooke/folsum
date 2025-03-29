// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::fs::File;
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::io::Write;
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::path::PathBuf;
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::sync::{Arc, Mutex, MutexGuard};
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::thread;

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// Internal crates macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use crate::{CSV_HEADERS, FoundFile};


/// Export the current summarization (show in the GUI table) to a FolSum CSV file.
///
/// # Parameters
/// - `export_file`: Path to the file that will be created.
/// - `file_paths`: Summarized files (from the GUI table).
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub fn export_csv(
    export_file: &Arc<Mutex<Option<PathBuf>>>,
    file_paths: &Arc<Mutex<Vec<FoundFile>>>,
) -> Result<(), &'static str> {
    // Copy extension counts so we can access them in a separate thread that's dedicated to this CSV dump.
    let file_paths_copy: Arc<Mutex<Vec<FoundFile>>> = file_paths.clone();
    // Copy the export file path's `Arc` so we can access it in a separate thread for CSV dumping.
    let export_filepath: Arc<Mutex<Option<PathBuf>>> = export_file.clone();

    thread::spawn(move || {
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
        // Lock the export file path so we can use it to create the CSV dump.
        let locked_export_file = export_filepath.lock().unwrap();

        let export_filename = locked_export_file
            .as_ref()
            .expect("No path for export file was specified");
        // Create a CSV file to write the extension types and their counts to, overwriting it if it already exists.
        let mut csv_export = File::create(export_filename).expect("Failed to create CSV export file");
        // Write the CSV's content to the export file.
        csv_export.write_all(csv_rows.as_bytes()).expect("Failed to write contents to CSV export file");
        info!("Exported file extension summary to: {:?}", export_filename);
    });
    Ok(())
}
