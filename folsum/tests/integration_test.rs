// Test-related dependencies.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use web_time::{Duration, Instant};

use folsum;


#[test]
fn test_summarization() {
    // Create the test directory in `./test-dir`.
    let test_dir = PathBuf::from("test_dir");
    // Mock some subdirectories that contain various files with different extensions.
    let answer_key = generate_mock_directory(&test_dir).unwrap();

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
    let unlocked_extension_counts = extension_counts.lock().unwrap();
    // For each file extension, ensure that the number of files found 
    for (found_extension, counts) in unlocked_extension_counts.iter() {
        assert_eq!(&answer_key[found_extension], counts);
    }
    // todo: Clean up test directories whether tests fail or succeed.
    // Cleanup: Recursively remove mocked subdirectories.
    let _delete_result = fs::remove_dir_all(&test_dir);
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
