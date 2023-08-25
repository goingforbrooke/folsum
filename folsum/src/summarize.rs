use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(not(target_arch = "wasm32"))]
use walkdir::WalkDir;
#[cfg(not(target_arch = "wasm32"))]
use web_time::{Duration, Instant};


pub fn summarize_directory(summarization_path: &Arc<Mutex<Option<PathBuf>>>,
                           extension_counts: &Arc<Mutex<HashMap<String, u32>>>,
                           summarization_start: &Arc<Mutex<Instant>>,
                           time_taken: &Arc<Mutex<Duration>>) -> Result<(), &'static str> {
    let unlocked_path: &mut Option<PathBuf> = &mut *summarization_path.lock().unwrap();
    // If the user picked a directory to summarize....
    if unlocked_path.is_some() {
        // ...then recursively count file extensions in the chosen directory.
        // Reset file extension counts to zero.
        *extension_counts.lock().unwrap() = HashMap::new();

        // Copy the Arcs of persistent members so they can be accessed by a separate thread.
        let extension_counts_copy = Arc::clone(&extension_counts);
        let summarization_path_copy = Arc::clone(&summarization_path);
        let start_copy = Arc::clone(&summarization_start);
        let time_taken_copy = Arc::clone(&time_taken);

        thread::spawn(move || {
            // Categorize extensionless files as "No extension."
            let default_extension = OsString::from("No extension");

            // Start the stopwatch for summarization time.
            let mut unlocked_start_copy = start_copy.lock().unwrap();
            *unlocked_start_copy = Instant::now();

            let unlocked_summarization_path = summarization_path_copy.lock().unwrap();
            // Clone the user's chosen path so we can release it's lock, allowing live table updates.
            let summarization_path_copy = unlocked_summarization_path.clone();
            // Release the mutex lock on the chosen path so extension count table can update.
            drop(unlocked_summarization_path);

            // Recursively iterate through each subdirectory and don't add subdirectories to the result.
            for entry in WalkDir::new(summarization_path_copy.unwrap())
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
            {
                // Extract the file extension from the file's name.
                let file_ext: &OsStr =
                    entry.path().extension().unwrap_or(&default_extension);
                let show_ext: String = String::from(file_ext.to_string_lossy());
                // Lock the extension counts variable so we can add a file to it.
                let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
                // Add newly encountered file extensions to known file extensions with a counter of 0.
                let counter: &mut u32 =
                    unlocked_counts_copy.entry(show_ext).or_insert(0);
                // Increment the counter for known file extensions by one.
                *counter += 1;
                // Update the summarization time stopwatch.
                let mut unlocked_time_taken_copy = time_taken_copy.lock().unwrap();
                *unlocked_time_taken_copy = unlocked_start_copy.elapsed();
            }
        });
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    // Test-related dependencies.
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use web_time::{Duration, Instant};

    use crate::summarize_directory;


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
        let _summarization_result = summarize_directory(&summarization_path,
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
}
