//! Verify an (in-memory) summarized directory against a verification file.
// Std crates for native and WASM builds.
use std::fs::{File, read_dir};
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::thread;

// Internal crates for native and WASM builds.
use crate::{CSV_HEADERS, FILEDATE_PREFIX_FORMAT, DirectoryVerificationStatus, FileIntegrity, IntegrityDetail, FoundFile};

// External crates for native and WASM builds.
use anyhow;
use anyhow::bail;
use chrono::NaiveDateTime;
#[allow(unused)]
use log::{debug, error, info, trace, warn};

/// Verify summarization against (CSV) verification file.
///
/// # Arguments
///
/// `manifest_file_path`: Path to a manifest file from a previous summarization.
///
/// # Returns
///
/// Which verification entries failed weren't found in the summary and why.
pub fn verify_summarization(summarized_files: &Arc<Mutex<Vec<FoundFile>>>,
                            summarization_path: &Arc<Mutex<Option<PathBuf>>>,
                            directory_verification_status: &Arc<Mutex<DirectoryVerificationStatus>>) -> Result<(), anyhow::Error> {
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let summarized_files = Arc::clone(&summarized_files);
    let summarization_path = Arc::clone(&summarization_path);
    let directory_verification_status = Arc::clone(&directory_verification_status);

    let _thread_handle = thread::spawn(move || {
        // Note that directory verification has begun.
        *directory_verification_status.lock().unwrap() = DirectoryVerificationStatus::InProgress;

        // Figure out which manifest file to verify against.
        let found_verification_manifests = find_verification_manifest_files(&summarization_path)?;
        let most_recent_manifest = decide_most_recent_manifest(&found_verification_manifests)?;

        let manifest_entries = load_previous_manifest(&most_recent_manifest.file_path)?;

        // todo: Relativize file path before verification steps b/c we're probably doing it twice.

        // Grab a file lock so we can filter for matching summarized files.
        let mut locked_summarized_files = summarized_files.lock().unwrap();

        // For each summarized file...
        for summarized_file in &mut locked_summarized_files.iter_mut() {
            // ... See if its file path exists in the verification manifest.
            let matching_manifest_entry = lookup_manifest_entry(&summarized_file.file_path, &manifest_entries)?;
            let assessed_integrity =  match matching_manifest_entry {
                Some(matching_manifest_entry) => {
                    // Assess the file's integrity (which is just an MD5) 😨.
                    assess_integrity(summarized_file, &matching_manifest_entry)?
                }
                None => {
                    // Assume bad file integrity b/c the file path wasn't found.
                    let assumed_integrity = IntegrityDetail::default();
                    FileIntegrity::VerificationFailed(assumed_integrity)
                }
            };

            // Modify shared memory entry for the summarized file-- add verification status (for column).
            match assessed_integrity {
                FileIntegrity::Verified(_) => summarized_file.file_verification_status = assessed_integrity,
                FileIntegrity::VerificationFailed(_) => summarized_file.file_verification_status = assessed_integrity,
                _ => {
                    let error_message = "Encountered unexpected integrity state {assessed_integrity:?} when only Verified or VerificationFailed was expected";
                    error!("{}", error_message);
                    bail!(error_message);
                }
            }
        }

        // Check if there were any verification failures.
        let verification_failures = locked_summarized_files.iter().any(|summarized_file| {
            matches!(summarized_file.file_verification_status, FileIntegrity::VerificationFailed(_))
        });
        // Note whether directory verification was successful in the GUI.
        if verification_failures {
            *directory_verification_status.lock().unwrap() = DirectoryVerificationStatus::VerificationFailed;
            info!("One or more summarized files failed verification")
        } else {
            *directory_verification_status.lock().unwrap() = DirectoryVerificationStatus::Verified;
            info!("Summarized files passed verification");
        }

        info!("Completed verification of summarized files");
        Ok(())
    });
    Ok(())
}

/// Look up a (recently-found) [`FoundFile`] summarization entry in a verification manifest from a previous run.
///
/// Files are found if their paths match.
fn lookup_manifest_entry(summarized_file_path: &PathBuf, manifest_entries: &Vec<FoundFile>) -> Result<Option<FoundFile>, anyhow::Error> {
    // Find entries from the verification file with paths that match this summarized file.
    let found_file = manifest_entries
        .iter()
        // Find every summarized file with a path that matches this verification entry.
        .find(|manifest_entry| {
            &manifest_entry.file_path == summarized_file_path
        })
        .cloned();

    // Log: Note what was found.
    match &found_file {
        Some(found_file) => trace!("Found a summarized file with a path in the verification manifest: {found_file:?}"),
        None => trace!("Found no summarized files with a path matching in the verification manifest were found."),
    };

    trace!("Found file in the verification manifest: {found_file:?}");
    Ok(found_file)
}

/// Decide if a file's integrity is intact (according to a previously-created manifest).
///
/// A [`FoundFile`] is considered verified if its relative path (to the root of the summarization directory) and hashes match.
fn assess_integrity(summarized_file: &FoundFile, manifest_entry: &FoundFile) -> Result<FileIntegrity, anyhow::Error> {
    // todo: note that file verification is "in progress" (for GUI column).
    let md5_hash_matches = &manifest_entry.md5_hash == &summarized_file.md5_hash;

    // Log: Note whether MD5 hashes match.
    match md5_hash_matches {
        true => trace!("MD5 hashes match"),
        false => trace!("MD5 hashes don't match")
    };

    let integrity_detail = IntegrityDetail {
        // We can safely assume that the file path has already been found.
        file_path_matches: true,
        md5_hash_matches,
    };

    // todo: Add SHA1 hashing.

    // Consider a file verified if the file path and MD5 hash match.
    let file_verification_status = match integrity_detail.file_path_matches && integrity_detail.md5_hash_matches {
        true => FileIntegrity::Verified(integrity_detail),
        false => FileIntegrity::VerificationFailed(integrity_detail),
    };

    debug!("Assessed integrity of manifest entry {manifest_entry:?} \
            and found it to be {file_verification_status:?}");
    Ok(file_verification_status)
}

/// Verification manifest from a previous run.
#[derive(Debug, Eq, PartialEq)]
struct VerificationManifest {
    file_path: PathBuf,
    date_created: NaiveDateTime,
}

impl VerificationManifest {
    fn new(file_path: &PathBuf, date_created: NaiveDateTime) -> Self {
        VerificationManifest {
            file_path: file_path.to_path_buf(),
            date_created,
        }
    }
}

/// Load [`FoundFile`]s from a verification (CSV) file.
fn load_previous_manifest(manifest_file_path: &PathBuf) -> Result<Vec<FoundFile>, anyhow::Error> {
    let csv_file_handle = File::open(&manifest_file_path)?;
    let mut line_iterator = io::BufReader::new(csv_file_handle).lines();

    // Ensure that the first line has the CSV headings that we expect.
    let first_line_content = match line_iterator.next() {
        Some(first_line) => first_line?,
        None => bail!("Found nothing in first line of file"),
    };
    // Remove the trailing newline in the header check b/c the line iterator does it too.
    match first_line_content == CSV_HEADERS.trim().to_string() {
        true => info!("Identified {manifest_file_path:?} as a valid FolSum CSV export"),
        false => bail!("The file {manifest_file_path:?} \
                        is an invalid FolSum CSV export. Found {first_line_content:?} \
                        when {CSV_HEADERS:?} was expected"),
    };

    let mut verification_entries: Vec<FoundFile> = vec![];
    // Interpret the remaining (non-header) CSV rows as file findings.
    for raw_line in line_iterator {
        let csv_line = raw_line?;

        // Ensure that the line has two items in it by checking for one comma.
        let comma_count = csv_line.chars().filter(|&character| character == ',').count();
        match comma_count {
            0 => bail!("Didn't find any items in the CSV row: {csv_line:?}"),
            1 => debug!("Ensured that two items are in the CSV row: {csv_line:?}"),
            _ => bail!("Found more than two items in the CSV row: {csv_line:?}"),
        }

        // Interpret CSV row as columns.
        let row_columns: Vec<&str> = csv_line.split(',').collect();
        let extracted_file_path = row_columns[0].trim();
        let extracted_md5_hash = row_columns[1].trim();

        let file_path = PathBuf::from(extracted_file_path);
        let md5_hash = extracted_md5_hash.to_string();
        let found_file = FoundFile::new(file_path, md5_hash);

        verification_entries.push(found_file);
    }

    let verification_entry_count = verification_entries.len();
    info!("Loaded {verification_entry_count:?} verification entries");
    Ok(verification_entries)
}

/// Find FolSum verification manifest files (in a summarized directory).
///
/// Assumes that manifest files are in the top-level directory.
fn find_verification_manifest_files(summarization_path: &Arc<Mutex<Option<PathBuf>>>) -> Result<Vec<VerificationManifest>, anyhow::Error> {
    let locked_summarization_path = summarization_path.lock().unwrap();
    let summarization_path_copy = locked_summarization_path.clone();
    drop(locked_summarization_path);

    // Assume that the user selected a summarization path b/c `verification_prerequisites_met` gates this function.
    let summarization_path = match summarization_path_copy {
        Some(summarization_path) => summarization_path,
        None => {
            let error_message = "Expected the user to have selected a summarization directory for us to find verification manifest files inside";
            error!("{error_message}");
            bail!(error_message);
        },
    };

    // Ensure that the summarization path is a directory before we try to look inside of it.
    if !summarization_path.is_dir() {
        let error_message = "Expected summarization path {summarization_path:?}\
                                  to be a directory";
        error!("{error_message}");
        bail!(error_message);
    }

    // Find CSV files in the summarization directory so we can log them before deeply checking the filename.
    let found_csv_files: Vec<PathBuf> = read_dir(&summarization_path)?
        // Ensure that directory entries are accessible..
        .filter_map(|found_entry| match found_entry {
            Ok(found_entry) => Some(found_entry.path()),
            Err(ref error_detail) => {
                // ... but don't error out if we can't access one.
                warn!("Failed to read directory entry {found_entry:?} \
                       while looking for verification manifest files in {summarization_path:?} \
                       due to error: {error_detail:?}");
                None
            }
        })
        // Filter for files.
        .filter(|found_entry| found_entry.is_file())
        // Filter for `.csv` extensions.
        .filter(|found_entry| {
            match found_entry.extension() {
                Some(actual_extension) => actual_extension == "csv",
                None => false,
            }
        })
        .collect();

    let csv_file_count = found_csv_files.len();
    debug!("Found {csv_file_count:?}\
            CSV files in the summarization directory {summarization_path:?}");

    // Find FolSum verification manifest files within the CSV files that we already found in the summarization directory.
    let found_manifest_paths: Vec<&PathBuf> = found_csv_files
        .iter()
        .filter(|found_csv_file| {
            let raw_csv_filename = found_csv_file.file_name().unwrap();
            let csv_filename = raw_csv_filename.to_string_lossy().to_string();
            csv_filename.ends_with(".folsum.csv")
        })
        .collect();

    // Convert manifest paths to an internal representation so they're easier to deal with.
    let found_manifest_files: Result<Vec<VerificationManifest>, anyhow::Error> = found_manifest_paths
        .iter()
        .map(|found_path| {
            let date_created = interpret_manifest_timestamp(found_path)?;
            Ok(VerificationManifest::new(found_path, date_created))
        })
        .collect();
    let found_manifest_files = found_manifest_files?;

    let found_file_count = found_manifest_files.len();
    info!("Found {found_file_count:?}: \
           {found_manifest_files:?}");
    Ok(found_manifest_files)
}

/// Decide which manifest file was most recently created.
///
/// # How We Decide Recency
///
/// 1. Extract the date from each verification file's name
/// 2. Keep the most recent date
fn decide_most_recent_manifest(found_verification_manifests: &Vec<VerificationManifest>) -> Result<&VerificationManifest, anyhow::Error> {
    let most_recent_manifest = found_verification_manifests
        .iter()
        .max_by_key(|verification_manifest| {
            verification_manifest.date_created
        });

    // Unpack the most recent
    let most_recent_manifest = match most_recent_manifest {
        Some(most_recent_manifest) => most_recent_manifest,
        // Bail if no manifest was found b/c that shouldn't be possible  at this point.
        None => {
            let error_message = "Expected to find at least one manifest while finding the most recent manifest";
            error!("{error_message}");
            bail!(error_message)
        }
    };

    info!("Decided that the most recent manifest in {found_verification_manifests:?} \
           is {most_recent_manifest:?}");
    Ok(most_recent_manifest)
}


/// Interpret an internal timestamp from a verification manifest file's name.
fn interpret_manifest_timestamp(manifest_path: &PathBuf) -> Result<NaiveDateTime, anyhow::Error> {
    let manifest_filename = match manifest_path.file_name() {
        Some(filename) => filename.to_string_lossy().to_string(),
        None => {
            let error_message = "Expected manifest file to have a name but found {manifest_path:?}";
            error!("{error_message}");
            bail!(error_message);
        },
    };

    // Extract the manifest's date from the file's prefix.
    let raw_date = match manifest_filename.split_once('_') {
        Some((date_prefix, _rest_of_filename)) => {
            date_prefix
        },
        None => {
            let error_message = format!("Expected a date prefix on FolSum manifest file {manifest_filename:?}");
            error!("{error_message}");
            bail!(error_message);
        }
    };

    trace!("Extracted raw date {raw_date:?} \
            from manifest filename {manifest_filename:?}");

    // Convert the raw date to something that we can work with.
    let interpreted_date = match NaiveDateTime::parse_from_str(raw_date, FILEDATE_PREFIX_FORMAT) {
        Ok(date_time) => date_time,
        Err(error_detail) => {
            let error_message = format!("Failed to interpret raw manifest file date {raw_date:?} \
                                               due to error {error_detail:?}");
            error!("{error_message}");
            bail!(error_message);
        },
    };

    trace!("Interpreted date for {manifest_path:?} \
            as {interpreted_date:?}");
    Ok(interpreted_date)
}

#[cfg(test)]
mod tests {
    use super::{find_verification_manifest_files, interpret_manifest_timestamp, VerificationManifest};

    use std::sync::{Arc, Mutex};

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use tempfile::Builder as TempfileBuilder;
    use tempfile::{NamedTempFile, tempdir};
    use pretty_assertions::assert_eq;
    use test_log::test;

    /// Ensure that [`find_verification_manifest_files`] finds manifest files without getting confused.
    #[test]
    fn test_find_manifest_files() -> Result<(), anyhow::Error> {
        // Test setup: create a temporary directory to put a mixture of verification files and misc files into.
        let temp_dir = tempdir()?;

        // What we want to find.
        let manifest_file = TempfileBuilder::new()
            .prefix("25-3-29-16-17_some_directory")
            .suffix(".folsum.csv")
            .tempfile_in(&temp_dir)?;

        // What we expect our finding to look like.
        let expected_path = manifest_file.path().to_path_buf();
        let expected_date = NaiveDate::from_ymd_opt(2025, 3, 29).unwrap();
        let expected_time = NaiveTime::from_hms_opt(16, 17, 0).unwrap();
        let expected_date_time = NaiveDateTime::new(expected_date, expected_time);
        let expected_manifest_finding = VerificationManifest::new(&expected_path, expected_date_time);

        // PDF that doesn't look like a manifest file at all.
        let non_manifest_file_pdf = TempfileBuilder::new()
            .prefix("portable_document_format")
            .suffix(".pdf")
            .tempfile_in(&temp_dir)?;

        // HTML that doesn't look like a manifest file at all.
        let non_manifest_file_html = TempfileBuilder::new()
            .prefix("hypertext_markup_language")
            .suffix(".html")
            .tempfile_in(&temp_dir)?;

        // Looks like a manifest file, but with no CSV extension.
        let false_manifest_file_no_csv = TempfileBuilder::new()
            .prefix("25-3-29-16-17_some_directory")
            .suffix(".folsum")
            .tempfile_in(&temp_dir)?;

        // Looks like a manifest file, but with no FolSum extension.
        let false_manifest_file_no_folsum = TempfileBuilder::new()
            .prefix("25-3-29-16-17_some_directory")
            .suffix(".csv")
            .tempfile_in(&temp_dir)?;

        // Looks like a manifest file, but is a directory, not a file.
        let false_manifest_file_is_dir = TempfileBuilder::new()
            .prefix("25-3-29-16-17_some_directory")
            .suffix(".folsum.csv")
            .tempdir_in(&temp_dir)?;

        // Set up shared memory for the test.
        let summarization_path = Arc::new(Mutex::new(Some(temp_dir.path().to_path_buf())));

        // Try finding the manifest file.
        let found_manifest_files = find_verification_manifest_files(&summarization_path)?;
        assert_eq!(found_manifest_files.len(), 1);
        assert!(found_manifest_files.contains(&expected_manifest_finding));

        Ok(())
    }

    /// Ensure that [`interpret_manifest_timestamp`] sees timestamps in verification filenames correctly.
    #[test]
    fn test_verification_manifest_timestamp_interpretation() -> Result<(), anyhow::Error> {
        // Set up the test.
        let manifest_file = TempfileBuilder::new()
            .prefix("25-3-29-16-17_some_directory")
            .suffix(".folsum.csv")
            .tempfile()?;
        let expected_date = NaiveDate::from_ymd_opt(2025, 3, 29).unwrap();
        let expected_time = NaiveTime::from_hms_opt(16, 17, 0).unwrap();
        let expected_interpretation = NaiveDateTime::new(expected_date, expected_time);

        let actual_interpretation = interpret_manifest_timestamp(&manifest_file.path().to_path_buf())?;

        assert_eq!(expected_interpretation, actual_interpretation);

        Ok(())
    }
}
