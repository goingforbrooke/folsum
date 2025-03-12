// Std crates for macOS, Windows, *and* WASM builds.
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

// External crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use walkdir::WalkDir;

// External crates for WASM builds.
#[cfg(target_family = "wasm")]
use web_time::{Duration, Instant};

// Internal crates for macOS, Windows, *and* WASM builds.
use crate::FoundFile;

/// Summarize directories in native builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
pub fn summarize_directory(
    summarization_path: &Arc<Mutex<Option<PathBuf>>>,
    file_paths: &Arc<Mutex<Vec<FoundFile>>>,
    summarization_start: &Arc<Mutex<Instant>>,
    time_taken: &Arc<Mutex<Duration>>,
) -> Result<(), &'static str> {
    let locked_path: &mut Option<PathBuf> = &mut *summarization_path.lock().unwrap();
    // If the user picked a directory to summarize....
    if locked_path.is_some() {
        // ...then recursively count file extensions in the chosen directory.

        // Reset file findings.
        *file_paths.lock().unwrap() = vec![FoundFile::default()];

        // Copy the Arcs of persistent members so they can be accessed by a separate thread.
        let file_paths_copy = Arc::clone(&file_paths);
        let summarization_path_copy = Arc::clone(&summarization_path);
        let start_copy = Arc::clone(&summarization_start);
        let time_taken_copy = Arc::clone(&time_taken);

        thread::spawn(move || {
            // Start the stopwatch for summarization time.
            let mut locked_start_copy = start_copy.lock().unwrap();
            *locked_start_copy = Instant::now();
            info!("Started summarization");

            let locked_summarization_path = summarization_path_copy.lock().unwrap();
            // Clone the user's chosen path so we can release it's lock, allowing live table updates.
            let summarization_path_copy = locked_summarization_path.clone();
            // Release the mutex lock on the chosen path so extension count table can update.
            drop(locked_summarization_path);

            match summarization_path_copy {
                Some(ref provided_path) => {
                    info!("Started recursing through {provided_path:?}");

                    // Recursively iterate through each subdirectory.
                    for entry in WalkDir::new(provided_path)
                        .min_depth(1)
                        .into_iter()
                        .filter_map(Result::ok)
                    {
                        debug!("Found file: {:?}", &entry.path());
                        // Extract the file extension from the file's name.
                        let file_path: PathBuf = entry.into_path();
                        // Lock the extension counts variable so we can add a file to it.
                        let mut locked_paths_copy = file_paths_copy.lock().unwrap();

                        let found_file = FoundFile {
                            file_path,
                            md5_hash: 0,
                        };

                        // Add newly encountered file paths to known file paths.
                        locked_paths_copy.push(found_file);

                        // Update the summarization time stopwatch.
                        let mut locked_time_taken_copy = time_taken_copy.lock().unwrap();
                        *locked_time_taken_copy = locked_start_copy.elapsed();
                    }
                },
                None => error!("No summarization path was provided"),
            }
        });
    };
    Ok(())
}

/// Summarize directories in WASM builds.
#[cfg(target_family = "wasm")]
pub fn wasm_demo_summarize_directory(
    file_paths: &Arc<Mutex<Vec<FoundFile>>>,
    summarization_start: &Arc<Mutex<Instant>>,
    time_taken: &Arc<Mutex<Duration>>,
    ) {
    // ...then recursively count file extensions in the chosen directory.

    // Reset file findings.
    *file_paths.lock().unwrap() = vec![FoundFile::default()];

    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let start_copy = Arc::clone(&summarization_start);
    let time_taken_copy = Arc::clone(&time_taken);
    let file_paths_copy = Arc::clone(&file_paths);

    // We usually do this in a separate thread, which `web_sys`'s (Web)workers would do a good job of simulating.
    // Temp: We skip this here because this demo's not dealing with actual files (or a large sample) anyway.
    // todo: add `web_sys` (Web)workers to WASM demo so the GUI doesn't hang for larger file counts.

    // Set up the browser demo by creating "fake files" to summarize.
    let actual_file_paths = generate_fake_file_paths(20, 3);

    // Start the stopwatch for summarization time.
    let mut locked_start_copy = start_copy.lock().unwrap();
    *locked_start_copy = Instant::now();
    info!("Started summarization");

    // Set up the demo by creating "fake files" to summarize.
    let actual_file_paths = generate_fake_file_paths(20, 3);

    // (Fake) WASM summarization.
    for file_path in actual_file_paths {
        // Pretend like a FoundFile for this item already existed.
        let found_file = FoundFile {file_path, md5_hash: 0};

        trace!("Found file: {:?}", &found_file);
        let mut locked_paths_copy= file_paths_copy.lock().unwrap();

        // Add newly encountered file paths to known file paths.
        locked_paths_copy.push(found_file);

        // Update the summarization time stopwatch.
        let mut locked_time_taken_copy = time_taken_copy.lock().unwrap();
        *locked_time_taken_copy = locked_start_copy.elapsed();
    }
}

// External test/demo crates.
#[cfg(any(debug_assertions, test, target_family = "wasm"))]
use rand::distr::Alphanumeric;
#[cfg(any(debug_assertions, test, target_family = "wasm"))]
use rand::{rng, Rng};

/// Create an "answer key" of fake file paths.
///
/// These will be used to create "fake files" for testing things like `summarize_directory`. This
/// fixture would be under `#[cfg(test)]`, but we need it for WASM builds so the browser demo has
/// something so summarize.
///
/// * `base_dir` - The root directory where the fake files will eventually be created.
/// * `total_files` - The total number of fake file paths to generate.
/// * `max_depth` - The maximum directory depth for the fake files.
/// * `extensions` - A slice of file extensions to randomly choose from.
#[cfg(any(debug_assertions, test, target_family = "wasm"))]
// Needs to be available for the browser demo and native unit tests (but not native builds).
#[allow(unused)]
fn generate_fake_file_paths(total_files: usize, max_depth: usize) -> Vec<PathBuf> {
    // Persist the random number generator to avoid re-initialization.
    let mut random_number_generator = rng();

    let mut fake_paths = Vec::new();

    // For each file that we need to create...
    for _ in 0..total_files {
        // Decide the depth for this file's directory.
        let current_depth = random_number_generator.random_range(0..=max_depth);

        let mut dir_paths = PathBuf::new();
        // Create a subdirectory at the current depth.
        for _ in 0..current_depth {
            // Create an eight character random dir name.
            let dir_name: String = (&mut random_number_generator)
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();
            // Add the new directory to the stack.
            dir_paths.push(dir_name);
        }

        // Create a ten character filename.
        let file_stem: String = rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        // File types for browser demos and unit tests.
        let file_extensions: Vec<&str> = vec!["pdf", "docx", "exe", "txt", "xlsx",
                                              "jpg", "png", "gif", "mp4", "avi",
                                              "mkv", "dll", "sys", "app", "dmg",
                                              "zip", "iso", "pages", "numbers",
                                              "7zip", "html", "py", "rs", "js",
                                              "rs"];

        // Choose a random extension from the provided list.
        let this_extension = file_extensions[rng().random_range(0..file_extensions.len())];

        // Create the full file name with extension.
        let file_name = format!("{}.{}", file_stem, this_extension);
        let file_path = dir_paths.join(file_name);
        fake_paths.push(file_path);
    }
    fake_paths
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, File};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use crate::summarize::{summarize_directory, generate_fake_file_paths};
    use crate::FoundFile;

    #[allow(unused)]
    use tracing::{debug, error, info, trace, warn};
    use tempfile::{tempdir, TempDir};

    /// Test fixture/demo setup: Create "fake files" to summarize in demos and unit tests.
    fn create_fake_files(desired_filepaths: &Vec<PathBuf>) -> Result<TempDir, anyhow::Error> {
        let temp_dir = tempdir().unwrap();

        for relative_testfile_path in desired_filepaths {
            // Put "faked files" in the temp dir so they're removed at the end of the test.
            let absolute_testfile_path: PathBuf = [temp_dir.as_ref(), relative_testfile_path].iter().collect();

            if let Some(file_parent) = absolute_testfile_path.parent() {
                create_dir_all(file_parent)?;
            }
            File::create(&absolute_testfile_path)?;

            debug!("Created test file: {absolute_testfile_path:?}");
        }
        // Return the tempdir handle so the calling function can keep it alive.
        Ok(temp_dir)
    }


    /// Native: Ensure that [`summarize_directory`] successfully finds directory contents.
    #[test_log::test]
    fn test_directory_summarization() -> Result<(), anyhow::Error> {
        // Set up the test by creating "fake files" to summarize.
        let actual_file_paths = generate_fake_file_paths(20, 3);
        let tempdir_handle = create_fake_files(&actual_file_paths)?;

        // Extract the tempdir containing the files to test against.
        let testdir_path = tempdir_handle.as_ref().to_path_buf();
        debug!("(Test) testdir_path = {:#?}", testdir_path);

        // Set up "dummy" datastores so we can run the test.
        let summarization_path = Arc::new(Mutex::new(Some(testdir_path)));
        let file_paths = Arc::new(Mutex::new(vec![FoundFile::default()]));
        let summarization_start = Arc::new(Mutex::new(Instant::now()));
        let time_taken = Arc::new(Mutex::new(Duration::ZERO));

        // Summarize the tempfiles.
        summarize_directory(&summarization_path, &file_paths, &summarization_start, &time_taken).unwrap();

        // Destroy the test files b/c we're done summarizing them.
        drop(tempdir_handle);

        // Lock the dummy file tracker so we can check its contents.
        let mut locked_paths_copy = file_paths.lock().unwrap();

        // Check if the summarization was successful.
        for found_file in locked_paths_copy.iter() {
            assert!(actual_file_paths.contains(&found_file.file_path));
            eprintln!("found_file = {:#?}", found_file);
        }

        Ok(())
    }
}