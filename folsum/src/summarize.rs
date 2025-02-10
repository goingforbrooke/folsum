// Std crates for macOS, Windows, *and* WASM builds.
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::thread;
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::time::{Duration, Instant};

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// External crates for macOS, and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use walkdir::WalkDir;
// External crates for WASM builds.
#[cfg(target_family = "wasm")]
use web_time::{Duration, Instant};


/// Summarize directories in macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
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

/// Summarize directories in WASM builds.
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


    // File extensions for our demo.
    let demo_file_extensions: Vec<&str> = vec!["pdf", "docx", "exe", "txt", "xlsx",
                                               "jpg", "png", "gif", "mp4", "avi",
                                               "mkv", "dll", "sys", "app", "dmg",
                                               "zip", "iso", "pages", "numbers",
                                               "7zip", "html", "py", "rs", "js",
                                               "rs"];

    // Generate numbers to sequentially assign as theoretical quantities of each file extension.
    let fibonacci_numbers = |n: usize| -> u32 {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            let temp = a;
            a = b;
            b = temp + b;
        }
        a
    };

    // Create fake (Fibonacci) counts for each file extension.
    let demo_file_counts: HashMap<&str, u32> = demo_file_extensions.iter().enumerate().map(|(index, item)| {
        let fib_num = fibonacci_numbers(index);
        (*item, fib_num)
    }).collect();

    // Create a file path for each the "fake file."
    let demo_file_paths: Vec<PathBuf> = demo_file_counts.iter().flat_map(|(file_extension, counter)| {
        let filename = format!("some_filename.{file_extension}");
        (0..*counter).map(move |_| PathBuf::from(&filename))
    }).collect();

    // Recursively iterate through each subdirectory and don't add subdirectories to the result.
    for entry in demo_file_paths {
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
