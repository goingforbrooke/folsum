use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;


pub fn export_csv(export_file: &Arc<Mutex<Option<PathBuf>>>, extension_counts: &Arc<Mutex<HashMap<String, u32>>>) -> Result<(), &'static str> {
    // Copy extension counts so we can access them in a separate thread that's dedicated to this CSV dump.
    let extension_counts_copy: Arc<Mutex<HashMap<String, u32>>> = Arc::clone(&extension_counts);
    // Copy the export file path's `Arc` so we can access it in a separate thread for CSV dumping.
    let export_file: Arc<Mutex<Option<PathBuf>>> = Arc::clone(&export_file);
    thread::spawn(move || {
        // Make a place to put extension counts that'll be written to the CSV file and include column headers.
        let mut csv_rows = String::from("File Extension, Occurrences\n");
        // Lock the extension counts so we can read them into CSV format.
        let unlocked_extension_counts = extension_counts_copy.lock().unwrap();
        for (extension_type, extension_count) in unlocked_extension_counts.iter() {
            // Ensure that there are no commas or newlines in this extension's name that would disrupt the output format.
            assert!(!extension_type.contains('\n') && !extension_type.contains(','));
            let csv_row = format!("{extension_type},{extension_count}\n");
            csv_rows.push_str(&csv_row)
        }
        // Lock the export file path so we can use it to create the CSV dump.
        let export_file = export_file.lock().unwrap();
        // Clone user's chosen export path so we can release it's lock, allowing live table updates.
        let export_file = export_file.clone().unwrap();
        // Create a CSV file to write the extension types and their counts to, overwriting it if it already exists.
        let mut csv_export = File::create(export_file).expect("Failed to create CSV export file");
        // Write the CSV's content to the export file.
        csv_export.write_all(csv_rows.as_bytes()).expect("Failed to write contents to CSV export file")
    });
    Ok(())
}

#[cfg(test)]
mod tests {

    // Test helpers.
    use crate::summarize::tests::generate_mock_directory;

    #[test]
    fn test_csv_export() {
        // Mock some subdirectories that contain various files with different extensions.
        let answer_key = generate_mock_directory(&test_dir).unwrap();
    }
}