// Test-related dependencies.
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use web_time::{Duration, Instant};

use folsum;


#[test]
fn test_summarization_and_export() {
    // Create the test directory in `./test-dir`.
    let test_dir = PathBuf::from("test_dir");
    // Mock some subdirectories that contain various files with different extensions.
    let actual_extensions = generate_mock_directory(&test_dir).unwrap();

    // Mock global state that's mutated by `summarize_directory`.
    let extension_counts = Arc::new(Mutex::new(HashMap::new()));
    let summarization_path = Arc::new(Mutex::new(Some(test_dir.clone())));
    let summarization_start = Arc::new(Mutex::new(Instant::now()));
    let time_taken = Arc::new(Mutex::new(Duration::ZERO));

    // Summarize the test directory so we can compare its output with the answer key.
    let _summarization_result = folsum::summarize_directory(&summarization_path,
                                                            &extension_counts,
                                                            &summarization_start,
                                                            &time_taken);
    // Wait a bit so the summarization thread has a chance to do it's thing.
    thread::sleep(Duration::from_secs(1));
    // Summarization test.
    for (found_extension, counts) in extension_counts.lock().unwrap().iter() {
        // For each summarized file extension, ensure that the number of files found matches the actual number of files.
        assert_eq!(&actual_extensions[found_extension], counts);
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
    
    let export_file = PathBuf::from("export_test.csv");
    // Mock the export filename as if the investigator named the file `export_test`
    let mocked_export_file = Arc::new(Mutex::new(Some(export_file.clone())));
    // Export summarization results of the mocked directory to CSV.
    let _export_result = folsum::export_csv(&mocked_export_file, &extension_counts);
    // todo: Check if the CSV export file exists first.
    // Extract header row from exported CSV.
    let exported_headers = read_csv_headers(export_file.clone()).unwrap();
    // Test if the CSV export headers are `File Extension` and `Occurrences`.
    assert_eq!(exported_headers, (String::from("File Extension"), String::from("Occurrences")));
    // Extract content rows from exported CSV.
    let exported_counts = read_csv_contents(export_file.clone());
    // For each file extension, ensure that the number of files found 
    for (summarized_extension, counts) in exported_counts.unwrap().iter() {
        // For each exported file extension, ensure that the number of files found matches the actual number of files.
        assert_eq!(&actual_extensions[summarized_extension], counts);
    }
    
    ///////////////////////////////////////////////////////////////////////////////////////////////
    
    // todo: Clean up mocked test directory whether tests fail or succeed.
    // Cleanup: Recursively remove mocked subdirectories.
    let _delete_result = fs::remove_dir_all(&test_dir);
}

fn read_csv_headers(export_file: PathBuf) -> io::Result<(String, String)> {
    let file = File::open(export_file).unwrap();
    let mut reader = BufReader::new(file);
    let mut reader_buffer = String::new();
    let csv_headers = reader.read_line(&mut reader_buffer)?.to_string();
    let mut parts = csv_headers.splitn(2, ',');
    let first_header = parts.next().unwrap();
    let second_header = parts.next().unwrap();
    Ok((first_header.to_string(), second_header.to_string()))
}

fn read_csv_contents(export_file: PathBuf) -> io::Result<HashMap<String, u32>> {
    let file = File::open(export_file)?;
    let reader = BufReader::new(file);
    let mut extension_counts: HashMap<String, u32> = HashMap::new();
    // Skip the first line in the CSV file because it's headers.
    for csv_line in reader.lines().skip(1) {
        let csv_line = csv_line?;
        // Separate each line on commas.
        let mut parts = csv_line.splitn(2, ',');
        // Assume that the extension name is the first part of the line.
        let extension_name = parts.next().unwrap().to_string();
        // Assume that the number of times the extension was seen is the first part of the line.
        let raw_occurrences: &str = parts.next().unwrap();
        let extension_occurrences: u32 = raw_occurrences.parse::<u32>().unwrap();
        extension_counts.insert(extension_name, extension_occurrences);
    }
    Ok(extension_counts)
}

pub fn generate_mock_directory(test_dir: &PathBuf) -> std::io::Result<HashMap<String, u32>> {
    let mut current_path = test_dir.clone();
    // Create a test directory in the current directory.
    fs::create_dir(&current_path)?;
    // Define the file extensions that'll be used to create (empty) test files.
    let extensions = vec!["py", "pdf", "doc", "zip", "xml"];
    // Keep track of how many files of each extension are created.
    let mut extension_counts: HashMap<String, u32> = HashMap::new();
    // Create subdirectories with a depth of ten.
    for subdir_depth in 1..=10 {
        // Name each subdirectory for its depth.
        current_path.push(format!("subdir_{}", subdir_depth));
        fs::create_dir(&current_path)?;
        // Create a number of empty files in each subdirectory equal to the subdirectory depth.
        for counter in 1..subdir_depth {
            // Pick a file extension for this file based off of how deep the subdirectory is.
            let extension = &extensions[counter % extensions.len()];
            let filename = format!("file_{}.{}", counter, extension);
            let file_path = current_path.join(filename);
            // Create the empty file.
            fs::File::create(file_path)?;
            // If the file extension already exists, then add one to it, otherwise, add it with "zero."
            *extension_counts.entry(extension.to_string()).or_insert(0) += 1;
        }
    }
    // Return the number of files created for each extension as a test "answer key."
    Ok(extension_counts)
}
