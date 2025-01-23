#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex, MutexGuard};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[allow(unused)]
use log::{debug, error, info, trace, warn};
#[cfg(not(target_arch = "wasm32"))]
use crate::sort_counts;

#[cfg(not(target_arch = "wasm32"))]
pub fn export_csv(
    export_file: &Arc<Mutex<Option<PathBuf>>>,
    extension_counts: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<(), &'static str> {
    // Copy extension counts so we can access them in a separate thread that's dedicated to this CSV dump.
    let extension_counts_copy: Arc<Mutex<HashMap<String, u32>>> = extension_counts.clone();
    // Copy the export file path's `Arc` so we can access it in a separate thread for CSV dumping.
    let export_file: Arc<Mutex<Option<PathBuf>>> = export_file.clone();
    thread::spawn(move || {
        // Make a place to put extension counts that'll be written to the CSV file and include column headers.
        let mut csv_rows = String::from("File Extension, Occurrences\n");
        // Lock extension counts so we can read them into CSV format.
        let locked_extension_counts: MutexGuard<'_, HashMap<String, u32>> =
            extension_counts_copy.lock().unwrap();
        // Sort extension counts by the number of occurrences (descending), then alphabetically (for extensions with the same count).
        let sorted_counts: Vec<(&String, &u32)> = sort_counts(&locked_extension_counts);
        for (extension_type, extension_count) in sorted_counts.iter() {
            // Ensure that there are no commas or newlines in this extension's name that would disrupt the output format.
            assert!(!extension_type.contains('\n') && !extension_type.contains(','));
            let csv_row = format!("{extension_type},{extension_count}\n");
            csv_rows.push_str(&csv_row)
        }
        // Lock the export file path so we can use it to create the CSV dump.
        let locked_export_file = export_file.lock().unwrap();
        let export_filename = locked_export_file
            .as_ref()
            .expect("No path for export file was specified");
        // Create a CSV file to write the extension types and their counts to, overwriting it if it already exists.
        let mut csv_export = File::create(export_filename).expect("Failed to create CSV export file");
        // Write the CSV's content to the export file.
        csv_export
            .write_all(csv_rows.as_bytes())
            .expect("Failed to write contents to CSV export file");
        info!("Exported file extension summary to: {:?}", export_filename);
    });
    Ok(())
}
