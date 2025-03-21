//! Verify an (in-memory) summarized directory against a verification file.
// Std crates for native and WASM builds.
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::thread;

// Internal crates for native and WASM builds.
use crate::{CSV_HEADERS, DirectoryVerificationStatus, FileIntegrity, IntegrityDetail, FoundFile};

// External crates for native and WASM builds.
use anyhow;
use anyhow::bail;
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
                            manifest_file_path: &Arc<Mutex<Option<PathBuf>>>,
                            directory_verification_status: &Arc<Mutex<DirectoryVerificationStatus>>) -> Result<(), anyhow::Error> {
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let summarized_files = Arc::clone(&summarized_files);
    let manifest_file_path = Arc::clone(&manifest_file_path);
    let directory_verification_status = Arc::clone(&directory_verification_status);

    let _thread_handle = thread::spawn(move || {
        // Note that directory verification has begun.
        *directory_verification_status.lock().unwrap() = DirectoryVerificationStatus::InProgress;

        let manifest_entries = load_previous_manifest(&manifest_file_path)?;

        // todo: Relativize file path before verification steps.

        // Grab a file lock so we can filter for matching summarized files.
        let mut locked_summarized_files = summarized_files.lock().unwrap();

        // For each summarized file...
        for summarized_file in &mut locked_summarized_files.iter_mut() {
            // ... See if its file path exists in the verification manifest.
            let matching_manifest_entry = lookup_manifest_entry(&summarized_file.file_path, &manifest_entries)?;
            let assessed_integrity =  match matching_manifest_entry {
                Some(matching_manifest_entry) => {
                    // Assess the file's integrity (which is just an MD5) 😨.
                    assess_integrity(&matching_manifest_entry, &matching_manifest_entry)?
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

    debug!("Assessed integrity of manifest entry {manifest_entry:?}");
    Ok(file_verification_status)
}

/// Load [`FoundFile`]s from a verification (CSV) file.
fn load_previous_manifest(manifest_file_path: &Arc<Mutex<Option<PathBuf>>>) -> Result<Vec<FoundFile>, anyhow::Error> {
    let locked_manifest_file_path = manifest_file_path.lock().unwrap();
    let manifest_file_path_copy = locked_manifest_file_path.clone();
    drop(locked_manifest_file_path);

    // Handle errors where no actual path was provided.
    let manifest_file_path_copy = match manifest_file_path_copy {
        Some(manifest_file_path_copy) => manifest_file_path_copy,
        None => {
            let error_message = "No manifest file path was provided";
            error!("{}", error_message);
            bail!(error_message);
        },
    };

    let csv_file_handle = File::open(&manifest_file_path_copy)?;
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
