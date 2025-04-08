//! Audit an (in-memory) directory in inventory against a manifest file.
//!
//! We accomplish this by comparing the manifest file against the directory's contents.
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{CSV_HEADERS, DirectoryVerificationStatus, FileIntegrity, FoundFile, IntegrityDetail, ManifestCreationStatus};

// External crates for native and WASM builds.
use anyhow;
use anyhow::bail;
use chrono::NaiveDateTime;
#[allow(unused)]
use log::{debug, error, info, trace, warn};

/// Audit directory inventory against a previously-generated (CSV) manifest file.
///
/// # Arguments
///
/// `manifest_file_path`: Path to a manifest file from a previous inventory.
///
/// # Returns
///
/// Manifest entries that weren't found in the directory inventory and why.
pub fn audit_summarization(summarized_files: &Arc<Mutex<Vec<FoundFile>>>,
                           directory_verification_status: &Arc<Mutex<DirectoryVerificationStatus>>,
                           manifest_creation_status: &Arc<Mutex<ManifestCreationStatus>>) -> Result<(), anyhow::Error> {
    // todo: Emit some kind of warning to the user if the manifest file's name doesn't match the directory's name.
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let summarized_files = Arc::clone(&summarized_files);
    let directory_verification_status = Arc::clone(&directory_verification_status);
    let manifest_creation_status = Arc::clone(&manifest_creation_status);

    let _thread_handle = thread::spawn(move || {
        // Note that directory verification has begun.
        *directory_verification_status.lock().unwrap() = DirectoryVerificationStatus::InProgress;

        // Extract the path to the previous manifest from the Arc.
        let locked_manifest_creation_status = manifest_creation_status.lock().unwrap();
        let manifest_creation_status_copy = locked_manifest_creation_status.clone();
        drop(locked_manifest_creation_status);
        let previous_manifest = match manifest_creation_status_copy {
            // Assume that a manifest file was already found b/c we checked prerequisites before this.
            ManifestCreationStatus::Done(manifest_path) => manifest_path,
            _ => {
                let error_message = "Expected a manifest file to be found";
                error!("{}", error_message);
                bail!(error_message);
            },
        };

        let manifest_entries = load_previous_manifest(&previous_manifest)?;

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationManifest {
    pub file_path: PathBuf,
    date_created: NaiveDateTime,
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
