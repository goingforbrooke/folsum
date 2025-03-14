//use std::collections::HashMap;
//use std::fs::{self, File};
//use std::io::{self, BufRead, BufReader};
//use std::path::PathBuf;
//use std::sync::{Arc, Mutex};
//use std::thread;
//
//use web_time::{Duration, Instant};
//
//use folsum;
//
//#[test]
//fn test_summarization_and_export() {
//    // Test Summarization /////////////////////////////////////////////////////////////////////////
//    // Create nested directories with empty test files.
//    let actual_extensions = TestFiles::new().unwrap();
//
//    // Mock global state variables that are mutated by `folsum::summarize_directory`.
//    let extension_counts = Arc::new(Mutex::new(HashMap::new()));
//    let summarization_path = Arc::new(Mutex::new(Some(actual_extensions.base_path.clone())));
//    let summarization_start = Arc::new(Mutex::new(Instant::now()));
//    let time_taken = Arc::new(Mutex::new(Duration::ZERO));
//
//    // Summarize the test directory so we can compare its output with the answer key.
//    let _summarization_attempt = folsum::summarize_directory(
//        &summarization_path,
//        &extension_counts,
//        &summarization_start,
//        &time_taken,
//    );
//    // Wait a bit so the summarization thread has a chance to do it's thing.
//    thread::sleep(Duration::from_secs(1));
//    // Test: Check if the file count for each summarized extension are accurate.
//    verify_extension_counts(&extension_counts.lock().unwrap(), &actual_extensions);
//    // Test CSV Export ////////////////////////////////////////////////////////////////////////////
//    let export_filename = &ExportFile::new().filename;
//    // Mock the export filename as if the investigator named the file `export_test`.
//    let mocked_export_file = Arc::new(Mutex::new(Some(export_filename.clone())));
//    // Export summarization results of the mocked directory to CSV.
//    let _export_attempt = folsum::export_csv(&mocked_export_file, &extension_counts);
//    // Wait a sec for the export to run so the export file exists before we try reading from it.
//    thread::sleep(Duration::from_secs(1));
//    // Test: Ensure that an export file was produced.
//    assert!(export_filename.exists());
//    // Extract header row from exported CSV.
//    let exported_headers = read_csv_headers(&export_filename).unwrap();
//    // Test if the CSV export headers are `File Extension` and `Occurrences`.
//    assert_eq!(
//        exported_headers,
//        (String::from("File Extension"), String::from("Occurrences"))
//    );
//    // Extract content rows from exported CSV, preserving their order.
//    let ordered_exported_counts: Vec<(String, u32)> = read_csv_contents(&export_filename).unwrap();
//    // Convert exported file extensions into a HashMap for efficient testing, disregarding their order.
//    let unordered_exported_counts: HashMap<String, u32> =
//        ordered_exported_counts.clone().into_iter().collect();
//    // Test: Check if the file count for each extension in the export is accurate.
//    verify_extension_counts(&unordered_exported_counts, &actual_extensions);
//    // Define the order that export file rows should be in: descending by count, then alphabetically.
//    let properly_sorted: Vec<(&String, &u32)> =
//        folsum::sort_counts(&actual_extensions.extension_counts);
//    // Test: Check if export file rows are ordered correctly: descending by count, then alphabetically.
//    for ((reported_extension, reported_count), (actual_extension, actual_count)) in
//        ordered_exported_counts.iter().zip(properly_sorted.iter())
//    {
//        assert_eq!(&reported_extension, actual_extension);
//        assert_eq!(&reported_count, actual_count);
//    }
//}
//
///// Test if the occurrences (the number of times a file with a given extension was encountered) for each
///// file extension is accurate.
//fn verify_extension_counts(
//    reported_extensions: &HashMap<String, u32>,
//    actual_extensions: &TestFiles,
//) {
//    // For each exported file extension...
//    for (reported_extension, reported_count) in reported_extensions.iter() {
//        // Look up the actual number of files with that extension.
//        let actual_count = &actual_extensions.extension_counts[reported_extension];
//        println!("Comparing \"{reported_count}\" occurrences of extension \"{reported_extension}\" to actual count of //\"{actual_count}\" occurrences");
//        // Ensure that the number of files found with that extension is correct.
//        assert_eq!(actual_count, reported_count);
//    }
//}
//
//fn read_csv_headers(export_file: &PathBuf) -> io::Result<(String, String)> {
//    let file = File::open(export_file)?;
//    let mut reader = BufReader::new(file);
//    let mut column_headers = String::new();
//    // Read a line of text into the buffer.
//    let _read_attempt = reader.read_line(&mut column_headers)?;
//    // Remove newline character from end of line.
//    let mut line_parts = column_headers.splitn(2, ',');
//    let first_header = line_parts.next().unwrap().trim();
//    let second_header = line_parts.next().unwrap().trim();
//    Ok((first_header.to_string(), second_header.to_string()))
//}
//
//fn read_csv_contents(export_file: &PathBuf) -> io::Result<Vec<(String, u32)>> {
//    let file = File::open(export_file)?;
//    let reader = BufReader::new(file);
//    let mut extension_counts: Vec<(String, u32)> = Vec::new();
//    // Skip the first line in the CSV file because it's headers.
//    for raw_line in reader.lines().skip(1) {
//        let csv_line = raw_line?;
//        // Separate each line on commas.
//        let mut parts = csv_line.splitn(2, ',');
//        // Assume that the extension name is the first part of the line.
//        let extension_name = parts.next().unwrap().to_string();
//        // Assume that the number of times the extension was seen is the first part of the line.
//        let raw_occurrences: &str = parts.next().unwrap();
//        // Convert extension count to an integer.
//        let extension_occurrences: u32 = raw_occurrences.parse::<u32>().unwrap();
//        extension_counts.push((extension_name, extension_occurrences));
//    }
//    Ok(extension_counts)
//}
//
///// Create nested subdirectories with empty files of various extensions in `test_dir/`.
//struct TestFiles {
//    // Create the test directory in `./test-dir`.
//    base_path: PathBuf,
//    // Remember the number of files created for each extension as a test "answer key."
//    extension_counts: HashMap<String, u32>,
//}
//
//impl TestFiles {
//    fn new() -> std::io::Result<Self> {
//        let base_path = PathBuf::from("test_dir");
//        let mut current_path = base_path.clone();
//        // Keep track of how many files of each extension are created.
//        let mut extension_counts: HashMap<String, u32> = HashMap::new();
//        // Create a test directory in the current directory.
//        fs::create_dir(&current_path)?;
//        // Define the file extensions that'll be used to create (empty) test files.
//        let extensions = vec!["py", "pdf", "doc", "zip", "xml"];
//        // Create subdirectories with a depth of ten.
//        for subdir_depth in 1..=10 {
//            // Name each subdirectory for its depth.
//            current_path.push(format!("subdir_{}", subdir_depth));
//            fs::create_dir(&current_path)?;
//            // Create a number of empty files in each subdirectory equal to the subdirectory depth.
//            for counter in 1..subdir_depth {
//                // Pick a file extension for this file based off of how deep the subdirectory is.
//                let extension = &extensions[counter % extensions.len()];
//                let filename = format!("file_{}.{}", counter, extension);
//                let file_path = current_path.join(filename);
//                // Create the empty file.
//                fs::File::create(file_path)?;
//                // If the file extension already exists, then add one to it, otherwise, add it with "zero."
//                *extension_counts.entry(extension.to_string()).or_insert(0) += 1;
//            }
//        }
//        println!(
//            "Created test directories with contents: {:?}",
//            extension_counts
//        );
//        Ok(Self {
//            base_path,
//            extension_counts,
//        })
//    }
//}
//
///// Whether the test using these directories passes or fails, delete them afterward.
//impl Drop for TestFiles {
//    fn drop(&mut self) {
//        let directory_path = self.base_path.clone();
//        // Recursively delete mocked subdirectories.
//        let _delete_result = fs::remove_dir_all(&directory_path);
//    }
//}
//
//struct ExportFile {
//    // Name the export file `export_test.csv`.
//    filename: PathBuf,
//}
//
//impl ExportFile {
//    fn new() -> Self {
//        let filename = PathBuf::from("export_test.csv");
//        Self { filename }
//    }
//}
//
//impl Drop for ExportFile {
//    fn drop(&mut self) {
//        let _delete_result = fs::remove_file(&self.filename);
//    }
//}
