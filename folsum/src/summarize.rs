use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

#[allow(unused)]
use log::{debug, error, info, trace, warn};
#[cfg(not(target_arch = "wasm32"))]
use walkdir::WalkDir;
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
pub fn summarize_directory(
    summarization_path: &Arc<Mutex<Option<PathBuf>>>,
    extension_counts: &Arc<Mutex<HashMap<String, u32>>>,
    summarization_start: &Arc<Mutex<Instant>>,
    time_taken: &Arc<Mutex<Duration>>,
) -> Result<(), &'static str> {
    let locked_path: &mut Option<PathBuf> = &mut *summarization_path.lock().unwrap();
    // If the user picked a directory to summarize....
    if locked_path.is_some() {
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
            let mut locked_start_copy = start_copy.lock().unwrap();
            *locked_start_copy = Instant::now();
            info!("Started summarization");

            let locked_summarization_path = summarization_path_copy.lock().unwrap();
            // Clone the user's chosen path so we can release it's lock, allowing live table updates.
            let summarization_path_copy = locked_summarization_path.clone();
            // Release the mutex lock on the chosen path so extension count table can update.
            drop(locked_summarization_path);

            // Recursively iterate through each subdirectory and don't add subdirectories to the result.
            for entry in WalkDir::new(summarization_path_copy.unwrap())
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
            {
                trace!("Found file: {:?}", &entry.path());
                // Extract the file extension from the file's name.
                let file_ext: &OsStr = entry.path().extension().unwrap_or(&default_extension);
                let show_ext: String = String::from(file_ext.to_string_lossy());
                // Lock the extension counts variable so we can add a file to it.
                let mut locked_counts_copy = extension_counts_copy.lock().unwrap();
                // Add newly encountered file extensions to known file extensions with a counter of 0.
                let counter: &mut u32 = locked_counts_copy.entry(show_ext).or_insert(0);
                // Increment the counter for known file extensions by one.
                *counter += 1;
                // Update the summarization time stopwatch.
                let mut locked_time_taken_copy = time_taken_copy.lock().unwrap();
                *locked_time_taken_copy = locked_start_copy.elapsed();
            }
        });
    };
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_demo_summarize_directory(
    extension_counts: &Arc<Mutex<HashMap<String, u32>>>,
    summarization_start: &Arc<Mutex<Instant>>,
    time_taken: &Arc<Mutex<Duration>>,
    ) {
    // ...then recursively count file extensions in the chosen directory.
    // Reset file extension counts to zero.
    *extension_counts.lock().unwrap() = HashMap::new();

    // Copy the Arcs of persistent members so they can be accessed by a separate thread. let extension_counts_copy = Arc::clone(&extension_counts);
    let start_copy = Arc::clone(&summarization_start);
    let time_taken_copy = Arc::clone(&time_taken);
    let extension_counts_copy = Arc::clone(&extension_counts);

    // We usually do this in a separate thread, which `web_sys`'s (Web)workers would do a good job of simulating.
    // We skip this here because this demo's not dealing with actual files (or a large sample) anyway.

    // Categorize extensionless files as "No extension."
    let default_extension = OsString::from("No extension");

    // Start the stopwatch for summarization time.
    let mut locked_start_copy = start_copy.lock().unwrap();
    *locked_start_copy = Instant::now();
    info!("Started summarization");

    //
    let demo_file_counts: Vec<(&str, u32)> = vec![("pdf", 10000),
                                                  ("docx", 10000),
                                                  ("exe", 10000),
                                                  ("txt", 10000),
                                                  ("xlsx", 10000),
                                                  ("jpg", 10000),
                                                  ("png", 10000),
                                                  ("gif", 10000),
                                                  ("mp4", 10000),
                                                  ("avi", 10000),
                                                  ("mkv", 10000),
                                                  ("dll", 10000),
                                                  ("sys", 10000),
                                                  ("app", 10000),
                                                  ("dmg", 10000),
                                                  ("zip", 10000),
                                                  ("iso", 10000)];

    // Create some demo files to summarize.
    let mut demo_files = vec![];
    for (file_extension, counter) in demo_file_counts.iter() {
        let filename = format!("some_filename.{file_extension}");
        for _count in 0u32..*counter {
            demo_files.push(PathBuf::from(&filename));
        }
    }

    // Recursively iterate through each subdirectory and don't add subdirectories to the result.
    for entry in demo_files {
        trace!("Found file: {:?}", &entry);
        // Extract the file extension from the file's name.
        let file_ext: &OsStr = entry.extension().unwrap_or(&default_extension);
        let show_ext: String = String::from(file_ext.to_string_lossy());
        // Lock the extension counts variable so we can add a file to it.
        let mut locked_counts_copy = extension_counts_copy.lock().unwrap();
        // Add newly encountered file extensions to known file extensions with a counter of 0.
        let counter: &mut u32 = locked_counts_copy.entry(show_ext).or_insert(0);
        // Increment the counter for known file extensions by one.
        *counter += 1;
        // Update the summarization time stopwatch.
        let mut locked_time_taken_copy = time_taken_copy.lock().unwrap();
        *locked_time_taken_copy = locked_start_copy.elapsed();
    }
}
