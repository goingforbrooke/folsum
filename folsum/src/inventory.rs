use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[allow(unused)]
use log::{debug, error, info, trace, warn};
use walkdir::WalkDir;

use crate::FoundFile;
use crate::get_md5_hash;
use crate::{DirectoryAuditStatus, ManifestCreationStatus, InventoryStatus};


/// Inventory a directory.
pub fn inventory_directory(
    chosen_inventory_path: &Arc<Mutex<Option<PathBuf>>>,
    inventoried_files: &Arc<Mutex<Vec<FoundFile>>>,
    inventory_start: &Arc<Mutex<Instant>>,
    time_taken: &Arc<Mutex<Duration>>,
    inventory_status: &Arc<Mutex<InventoryStatus>>,
    directory_audit_status: &Arc<Mutex<DirectoryAuditStatus>>,
    manifest_creation_status: &Arc<Mutex<ManifestCreationStatus>>,
) -> Result<(), &'static str> {

    let locked_chosen_inventory_path: &mut Option<PathBuf> = &mut *chosen_inventory_path.lock().unwrap();
    // If the user picked a directory to inventory....
    if locked_chosen_inventory_path.is_some() {
        // Reset file findings.
        *inventoried_files.lock().unwrap() = vec![];

        // Note that inventory is in progress.
        *inventory_status.lock().unwrap() = InventoryStatus::InProgress;
        *directory_audit_status.lock().unwrap() = DirectoryAuditStatus::Unaudited;
        *manifest_creation_status.lock().unwrap() = ManifestCreationStatus::NotStarted;

        // Copy the Arcs of persistent members so they can be accessed by a separate thread.
        let chosen_inventory_path_copy = Arc::clone(&chosen_inventory_path);
        let inventoried_files_copy = Arc::clone(&inventoried_files);
        let start_copy = Arc::clone(&inventory_start);
        let time_taken_copy = Arc::clone(&time_taken);
        let inventory_status_copy = Arc::clone(&inventory_status);

        thread::spawn(move || {
            // Start the stopwatch for inventory time.
            let mut locked_start_copy = start_copy.lock().unwrap();
            *locked_start_copy = Instant::now();
            info!("Started inventory");

            let locked_inventory_path = chosen_inventory_path_copy.lock().unwrap();
            // Clone the user's chosen path so we can release its lock, allowing live table updates.
            let inventory_path_copy = locked_inventory_path.clone();
            // Release the mutex lock on the chosen path so the inventory table in the GUI can update.
            drop(locked_inventory_path);

            match inventory_path_copy {
                Some(ref provided_path) => {
                    info!("Started recursing through {provided_path:?}");

                    // Recursively iterate through each subdirectory.
                    for dir_entry in WalkDir::new(provided_path)
                        // Don't consider the top-level directory as an item.
                        .min_depth(1)
                        .into_iter()
                        .filter_map(Result::ok)
                        // Ignore subdirectories at all depths.
                        .filter(|dir_entry| !dir_entry.file_type().is_dir())
                    {
                        let foundfile_path: PathBuf = dir_entry.into_path();
                        debug!("Found directory (file) entry: {foundfile_path:?}");

                        // Convert from absolute path to a relative (to given directory) path.
                        // todo: Handle relative path prefix strip errors.
                        let file_path = foundfile_path.strip_prefix(provided_path).unwrap().to_path_buf();
                        // todo: Propagate errors for "No such file or directory" when running `get_md5_hash` in `inventory_directory`.
                        let md5_hash = get_md5_hash(&foundfile_path).unwrap();
                        let found_file = FoundFile::new(file_path, md5_hash);

                        // Lock the extension counts variable so we can add a file to it.
                        let mut locked_paths_copy = inventoried_files_copy.lock().unwrap();

                        // Add newly encountered file paths to known file paths.
                        locked_paths_copy.push(found_file);

                        // Release the file paths lock so the GUI can update.
                        drop(locked_paths_copy);

                        // Update the inventory time stopwatch.
                        let mut locked_time_taken_copy = time_taken_copy.lock().unwrap();
                        *locked_time_taken_copy = locked_start_copy.elapsed();
                    }
                    // End of loop
                },
                None => error!("No inventory path was provided"),
            }
            *inventory_status_copy.lock().unwrap() = InventoryStatus::Done;
        });
    };
    Ok(())
}

#[cfg(any(test, feature = "bench"))]
pub mod tests {
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use crate::common::{DirectoryAuditStatus, ManifestCreationStatus, InventoryStatus};
    use crate::hashers::get_md5_hash;
    use crate::FoundFile;
    use crate::inventory::inventory_directory;

    #[cfg(test)]
    use anyhow::bail;
    #[cfg(feature = "bench")]
    use rand::distr::Alphanumeric;
    #[cfg(feature = "bench")]
    use rand::{rng, Rng};
    use test_log;
    use tempfile::{tempdir, TempDir};
    #[allow(unused)]
    use tracing::{debug, error, info, trace, warn};


    /// Create an "answer key" of fake file paths.
    ///
    /// These will be used to create "fake files" for testing things like `audit_directory`.
    ///
    /// * `total_files` - The total number of fake file paths to generate.
    /// * `max_depth` - The maximum directory depth for the fake files.
    pub fn generate_fake_file_paths(total_files: u32, max_depth: u16) -> Vec<PathBuf> {
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

    /// Test fixture/demo setup: Create "fake files" to inventory in demos and unit tests.
    fn create_fake_files(desired_filepaths: &Vec<PathBuf>) -> Result<TempDir, anyhow::Error> {
        let temp_dir = tempdir().unwrap();

        for relative_testfile_path in desired_filepaths {
            // Put "faked files" in the temp dir so they're removed at the end of the test.
            let absolute_testfile_path: PathBuf = [temp_dir.as_ref(), relative_testfile_path].iter().collect();

            if let Some(file_parent) = absolute_testfile_path.parent() {
                create_dir_all(file_parent)?;
            }

            // Get an RNG:
            let rng = rand::rng();
            // Generate 100 random characters to put in the fake file so each MD5 hash is different.
            let random_character_bytes: Vec<u8> = rng
                .sample_iter(&rand::distr::Alphanumeric)
                .take(100)
                .collect();

            let mut buffer = File::create(&absolute_testfile_path)?;
            buffer.write_all(&random_character_bytes)?;

            debug!("Created test file: {absolute_testfile_path:?}");
        }
        // Return the tempdir handle so the calling function can keep it alive.
        Ok(temp_dir)
    }

    /// Test fixture/demo setup: Create "fake MD5 hashes" of fake files to validate integrity checking mechanisms.
    fn create_fake_md5_hashes(root_dir: &PathBuf, desired_filepaths: &Vec<PathBuf>) -> Result<Vec<String>, anyhow::Error> {
        let mut expected_hashes: Vec<String> = vec![];
        for relative_testfile_path in desired_filepaths {
            // Put "faked files" in the temp dir so they're removed at the end of the test.
            let absolute_testfile_path: PathBuf = [root_dir, relative_testfile_path].iter().collect();

            // Assume that MD5 hashing works b/c that function has its own unit test.
            let actual_md5_hash = get_md5_hash(&absolute_testfile_path)?;
            debug!("Hashed test file: {absolute_testfile_path:?}");

            expected_hashes.push(actual_md5_hash);
        }
        // Return the tempdir handle so the calling function can keep it alive.
        Ok(expected_hashes)
    }

    /// Perform inventory in a temporary directory of "fake" files.
    ///
    /// This is abstracted away from [`test_directory_audit`]... so it can be called by the benchmarker.
    ///
    /// # Returns
    ///
    /// Tuple:
    /// - datastore variable (to check at the end of a test)
    /// - `Vec<String>` of MD5 hashes that we expect to find.
    pub fn perform_fake_inventory(expected_file_paths: &Vec<PathBuf>) -> Result<(Arc<Mutex<Vec<FoundFile>>>, Vec<String>), anyhow::Error> {
        let tempdir_handle = create_fake_files(&expected_file_paths)?;
        // Extract the tempdir containing the files to test against.
        let testdir_path = tempdir_handle.as_ref().to_path_buf();
        debug!("(Test) testdir_path = {:#?}", testdir_path);

        let expected_md5_hashes = create_fake_md5_hashes(&testdir_path, &expected_file_paths)?;


        // Set up "dummy" datastores so we can run the test.
        let chosen_inventory_path = Arc::new(Mutex::new(Some(testdir_path)));
        let file_paths = Arc::new(Mutex::new(vec![]));
        let inventory_start = Arc::new(Mutex::new(Instant::now()));
        let time_taken = Arc::new(Mutex::new(Duration::ZERO));
        let inventory_status = Arc::new(Mutex::new(InventoryStatus::NotStarted));
        let directory_audit_status = Arc::new(Mutex::new(DirectoryAuditStatus::Unaudited));
        let manifest_creation_status = Arc::new(Mutex::new(ManifestCreationStatus::NotStarted));

        // Inventory the tempfiles.
        inventory_directory(&chosen_inventory_path,
                            &file_paths,
                            &inventory_start,
                            &time_taken,
                            &inventory_status,
                            &directory_audit_status,
                            &manifest_creation_status).unwrap();

        // Keep the test files around long enough for inventory to finish.
        loop {
            if matches!(*inventory_status.lock().unwrap(), InventoryStatus::Done) {
                // Destroy the test files b/c we're done inventory them.
                drop(tempdir_handle);
                break;
            }
            sleep(Duration::from_millis(50))
        }


        // Return the datastore variable so the unit test can verify what's been inventoried.
        Ok((file_paths, expected_md5_hashes))
    }

    ///// Ensure that [`inventory_directory`] doesn't include FolSum manifest files in its findings.
    //#[test_log::test]
    //fn test_manifest_files_are_ignored() -> Result<(), anyhow::Error> {

    //}

    /// Ensure that [`inventory_directory`] successfully finds directory contents.
    ///
    /// Assumes a scenario in which all files exist and have valid integrity.
    #[test_log::test]
    fn test_directory_inventory_integrity_valid() -> Result<(), anyhow::Error> {
        // Set up the test.
        let expected_file_paths = generate_fake_file_paths(20, 3);

        let (file_paths, expected_md5_hashes) = perform_fake_inventory(&expected_file_paths)?;

        // Assume that inventory will complete in less than a second.
        sleep(Duration::from_secs(1));

        // Lock the dummy file tracker so we can check its contents.
        let locked_paths_copy = file_paths.lock().unwrap();

        // Check if inventory was successful.
        for actual_found_file in locked_paths_copy.iter() {
            let actual_file_path = &actual_found_file.file_path;
            assert!(expected_file_paths.contains(actual_file_path),
                    "Expected to find {actual_file_path:?} \
                     in {expected_file_paths:?}");
            let actual_md5_hash = &actual_found_file.md5_hash;
            assert!(expected_md5_hashes.contains(actual_md5_hash),
                    "Expected to find {actual_file_path:?} \
                     in {expected_file_paths:?}");
        }
        Ok(())
    }

    /// Native: Ensure that [`inventory_directory`] successfully finds audit discrepancies.
    ///
    /// Assumes a scenario in which all files exist, but one's MD5 hash has been perturbed.
    #[test_log::test]
    fn test_directory_inventory_integrity_invalid() -> Result<(), anyhow::Error> {
        // Set up the test.
        let expected_file_paths = generate_fake_file_paths(20, 3);

        let (file_paths, mut expected_md5_hashes) = perform_fake_inventory(&expected_file_paths)?;

        // Keep around the original hash so we can ensure that it was missed later.
        let pre_perturbed_hash = expected_md5_hashes.first().unwrap().clone();
        // Perturbation: Mess up the first MD5 hash, as if the manifest file showed something different from what will be inventoried, b/c we want to catch that!
        *expected_md5_hashes.first_mut().unwrap() = "😱😱😱😱😱😱😱😱😱😱😱😱😱😱😱😱😱😱".to_string();

        // Assume that inventory will complete in less than a second.
        sleep(Duration::from_secs(1));

        // Lock the dummy file tracker so we can check its contents.
        let locked_paths_copy = file_paths.lock().unwrap();

        // Keep track of our little assertions so we can see if anything failed at the end.
        let mut existence_check_failures: Vec<&PathBuf> = vec![];
        let mut hash_match_failures: Vec<&String> = vec![];
        // Check if the inventory was successful (
        for actual_found_file in locked_paths_copy.iter() {
            // Check if the file paths match.
            let actual_file_path = &actual_found_file.file_path;
            if !expected_file_paths.contains(actual_file_path) {
                existence_check_failures.push(&actual_file_path);
            }

            // Check if the MD5 hashes match.
            let actual_md5_hash = &actual_found_file.md5_hash;
            if !expected_md5_hashes.contains(actual_md5_hash) {
                hash_match_failures.push(actual_md5_hash);
            }
        }

        assert!(existence_check_failures.is_empty(),
                "Didn't find file_path {existence_check_failures:?} \
                 in {expected_file_paths:?}");

        eprintln!("pre_perturbed_hash = {:#?}", pre_perturbed_hash);
        // Now for the actual test-- is FolSum sad that it missed the perturbed hash?
        if !hash_match_failures.is_empty() {
            // Happy path: FolSum notices that one of the MD5 hashes was messed with.
            if hash_match_failures.len() == 1 {
                let hash_match_failure = hash_match_failures.first().cloned().unwrap();
                // Ensure that the messed up hash is the one that we perturbed.
                assert!(pre_perturbed_hash == *hash_match_failure,
                        "Expected the perturbed hash to be {pre_perturbed_hash:?} \
                         but found {hash_match_failure:?} \
                         instead.");
            } else {
                let failure_count = hash_match_failures.len();
                bail!("Expected to find only one hash match failure, but {failure_count:?}\
                       hash match failures were found")
            }
        } else {
            bail!("Didn't find hash {hash_match_failures:?} \
                   in {expected_md5_hashes:?}")
        }
        Ok(())
    }
}