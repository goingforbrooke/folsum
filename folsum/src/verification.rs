//! Verify an (in-memory) summarized directory against a verification file.
// Std crates for native and WASM builds.
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Internal crates for native and WASM builds.
use crate::{CSV_HEADERS, FoundFile, get_md5_hash};

// External crates for native and WASM builds.
use anyhow;
use anyhow::bail;
#[allow(unused)]
use log::{debug, error, info, trace, warn};

/// Verify summarization against (CSV) verification file.
pub fn verify_summarization(summarized_files: &Arc<Mutex<Vec<FoundFile>>>,
                            verification_file_path: &Arc<Mutex<Option<PathBuf>>>,
                            summarization_path: &Arc<Mutex<Option<PathBuf>>>, ) -> Result<Vec<(FoundFile, FileVerificationStatus)>, anyhow::Error> {
    let locked_verification_file_path = verification_file_path.lock().unwrap();
    let verification_file_path_copy = locked_verification_file_path.clone();
    drop(locked_verification_file_path);
    let verification_entries = match verification_file_path_copy {
        Some(verification_file_path) => {
            load_verification_entries(&verification_file_path)?
        },
        None => bail!("No verification file"),
    };

    // todo: Figure out what to do if something more than what's in the verification file is encountered.

    let mut verification_failures: Vec<(FoundFile, FileVerificationStatus)> = vec![];
    // For each previously-found entry from the verification file...
    for verification_entry in verification_entries {
        // ... check if there's a matching summarization file.
        let verification_status = verify_file(&verification_entry, summarized_files, summarization_path)?;
        // If something doesn't match...
        if !verification_status.file_path_matches || !verification_status.md5_hash_matches {
            // ... then note it.
            verification_failures.push((verification_entry, verification_status))
        }
    }

    if verification_failures.is_empty() {
        info!("Summarized files passed verification");
    } else {
        let failure_count = verification_failures.len();
        info!("Found {failure_count:?} summarized files failed verification")
    }

    info!("Completed verification");
    Ok(verification_failures)
}

#[derive(Debug, Default)]
pub struct FileVerificationStatus {
    file_path_matches: bool,
    md5_hash_matches: bool,
}

/// Look up a previously-found [`FoundFile`] verification entry in the summarized output.
///
/// A [`FoundFile`] is considered verified if its relative path (to the root of the summarization directory) and hashes match.
fn verify_file(verification_entry: &FoundFile,
               summarized_files: &Arc<Mutex<Vec<FoundFile>>>,
               summarization_path: &Arc<Mutex<Option<PathBuf>>>) -> Result<FileVerificationStatus, anyhow::Error> {
    // Grab a file lock so we can filter for matching summarized files.
    let locked_summarized_files = summarized_files.lock().unwrap();
    // Find entries from the verification file with paths that match this summarized file.
    let with_matching_paths: Vec<FoundFile> = locked_summarized_files
        .iter()
        .filter(|verification_entry| {
            verification_entry.file_path == verification_entry.file_path
        })
        // Clone matches b/c we need to release the lock and the memory cost is tiny.
        .cloned()
        .collect();
    // Drop the lock so the GUI can update.
    drop(locked_summarized_files);

    let file_path_matches = match with_matching_paths.len() {
        0 => {
            let verification_entry_path = &verification_entry.file_path;
            trace!("No summarized files with a path matching the verification entry {verification_entry_path:?} were found.");
            false
        },
        1 => {
            trace!("Found a summarized file with a path the verification entry: {with_matching_paths:?}");
            true
        },
        _ => {
            let verification_entry_path = &verification_entry.file_path;
            // todo: Figure out what to do if more than one matching paths were found during verification.
            bail!("Found more than one summarized file with a path matching the verification entry {verification_entry_path:?} {with_matching_paths:?}");
        }
    };

    let md5_hash_matches = match file_path_matches {
        true => {
            // Get the path to the summarization directory so we can use it to build absolute paths (for MD5 hashing).
            let locked_summarization_path = summarization_path.lock().unwrap();
            let summarization_path_copy = locked_summarization_path.clone().expect("Expected a summarization path to be chosen before verification began");
            // Release the mutex lock on the summarization path so the summarization count table can update.
            drop(locked_summarization_path);

            let relative_path = &verification_entry.file_path;

            // Build an absolute path for MD5 hashing.
            let absolute_path: PathBuf = [&summarization_path_copy, relative_path].iter().collect();
            let actual_md5_hash = get_md5_hash(&absolute_path)?;

            &verification_entry.md5_hash == &actual_md5_hash
        },
        // MD5 hashes automatically don't match if the file path doesn't match.
        false => false,
    };

    if md5_hash_matches {
        trace!("MD5 hashes match");
    } else {
        trace!("MD5 hashes don't match");
    }

    let verification_status = FileVerificationStatus {
        file_path_matches,
        md5_hash_matches,
    };

    Ok(verification_status)
}

/// Load [`FoundFile`]s from a verification (CSV) file.
fn load_verification_entries(verification_file_path: &PathBuf) -> Result<Vec<FoundFile>, anyhow::Error> {
    let csv_file_handle = File::open(verification_file_path)?;
    let mut line_iterator = io::BufReader::new(csv_file_handle).lines();

    // Ensure that the first line has the CSV headings that we expect.
    let first_line_content = match line_iterator.next() {
        Some(first_line) => first_line?,
        None => bail!("Found nothing in first line of file"),
    };
    // Remove the trailing newline in the header check b/c the line iterator does it too.
    match first_line_content == CSV_HEADERS.trim().to_string() {
        true => info!("Identified {verification_file_path:?} as a valid FolSum CSV export"),
        false => bail!("The file {verification_file_path:?} \
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

        let found_file = FoundFile {
            file_path: PathBuf::from(extracted_file_path),
            md5_hash: extracted_md5_hash.to_string(),
        };

        verification_entries.push(found_file);
    }

    info!("Loaded verification entries");
    Ok(verification_entries)
}
