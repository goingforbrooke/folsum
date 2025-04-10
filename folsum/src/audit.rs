//! Audit an (in-memory) directory inventory against a manifest file.
//!
//! We accomplish this by comparing the manifest file's listings against the directory's contents.
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{CSV_HEADERS, DirectoryAuditStatus, FileIntegrity, FoundFile, FileIntegrityDetail};

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
/// - `inventoried_files`: Inventory of a directory's contents.
/// - `directory_audit_status`: Where we are in the audit process.
/// - `manifest_creation_status`: Where we are in the manifest creation process.
///
/// # Returns
///
/// Manifest entries that weren't found in the directory inventory and why.
pub fn audit_directory_inventory(inventoried_files: &Arc<Mutex<Vec<FoundFile>>>,
                                 directory_audit_status: &Arc<Mutex<DirectoryAuditStatus>>,
                                 chosen_manifest: &Arc<Mutex<Option<PathBuf>>>) -> Result<(), anyhow::Error> {
    // todo: Emit some kind of warning to the user if the manifest file's name doesn't match the directory's name.
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let inventoried_files = Arc::clone(&inventoried_files);
    let directory_audit_status = Arc::clone(&directory_audit_status);
    let chosen_manifest = Arc::clone(&chosen_manifest);

    let _thread_handle = thread::spawn(move || {
        // Note that directory audit has begun.
        *directory_audit_status.lock().unwrap() = DirectoryAuditStatus::InProgress;

        let locked_chosen_manifest = chosen_manifest.lock().unwrap();
        let chosen_manifest_copy = locked_chosen_manifest.clone();
        drop(locked_chosen_manifest);
        let chosen_manifest_path = match chosen_manifest_copy {
            Some(chosen_manifest_path) => chosen_manifest_path,
            None => {
                let error_message = "Expected to find a chosen manifest";
                error!("{}", error_message);
                bail!(error_message)
            }
        };

        let manifest_entries = load_previous_manifest(&chosen_manifest_path)?;

        // todo: Relativize file path before audit steps b/c we're probably doing it twice.

        // Grab a file lock so we can filter for matching inventoried files.
        let mut locked_inventoried_files = inventoried_files.lock().unwrap();

        // Check each inventoried file against the manifest b/c we assume that most files will exist.
        for inventoried_file in &mut locked_inventoried_files.iter_mut() {
            // ... See if its file path exists in the manifest.
            let matching_manifest_entry = lookup_manifest_entry(&inventoried_file.file_path, &manifest_entries)?;

            let assessed_integrity = match matching_manifest_entry {
                // If the inventoried file exists in the manifest, then assess the file's integrity (which is just an MD5) 😨.
                Some(matching_manifest_entry) => assess_integrity(inventoried_file, &matching_manifest_entry)?,
                // If the inventoried file doesn't exist in the manifest then the inventoried file was added.
                None => FileIntegrity::NewlyAdded,
            };

            // Modify shared memory entry for the inventoried file so we can show the audit status in its respective column.
            match assessed_integrity {
                FileIntegrity::Verified(_) => inventoried_file.file_integrity = assessed_integrity,
                FileIntegrity::VerificationFailed(_) => inventoried_file.file_integrity = assessed_integrity,
                _ => {
                    let error_message = format!("Encountered unexpected integrity state {assessed_integrity:?}\
                                                       when only Verified or VerificationFailed was expected");
                    error!("{}", error_message);
                    bail!(error_message);
                }
            }
        }

        // Sanity check: nothing should be unexamined.
        let unexamined_files: Vec<&FoundFile> = locked_inventoried_files.iter()
            .filter(|found_file| {
                matches!(found_file.file_integrity, FileIntegrity::Unverified)
            })
            .collect();
        if !unexamined_files.is_empty() {
            let unexamined_count = unexamined_files.len();
            warn!("Encountered {unexamined_count} \
                   unexamined files: {unexamined_files:?}");
        }

        // Check if there were any audit failures.
        let audit_failures = locked_inventoried_files.iter().any(|found_file| {
            matches!(found_file.file_integrity, FileIntegrity::VerificationFailed(_))
        });
        // Note whether directory audit was successful in the GUI.
        if audit_failures {
            *directory_audit_status.lock().unwrap() = DirectoryAuditStatus::DiscrepanciesFound;
            info!("One or more inventoried files failed audit")
        } else {
            *directory_audit_status.lock().unwrap() = DirectoryAuditStatus::Audited;
            info!("Inventoried files passed audit");
        }

        info!("Completed audit of inventoried files");
        Ok(())
    });
    Ok(())
}

/// Look up a (recently-found) [`FoundFile`] inventory entry in a FolSum manifest from a previous run.
///
/// Files are found if their paths match.
fn lookup_manifest_entry(inventoried_file_path: &PathBuf,
                         manifest_entries: &Vec<FoundFile>) -> Result<Option<FoundFile>, anyhow::Error> {
    // Find entries from the manifest file with paths that match this inventoried file.
    let found_file = manifest_entries
        .iter()
        // Find every inventoried file with a path that matches this manifest entry.
        .find(|manifest_entry| {
            &manifest_entry.file_path == inventoried_file_path
        })
        .cloned();

    // Log: Note what was found.
    match &found_file {
        Some(found_file) => trace!("Found a inventoried file with a path in the manifest: {found_file:?}"),
        None => trace!("Found no inventoried files with a matching path in the manifest."),
    };

    debug!("Found a file with a matching path in the manifest: {found_file:?}");
    Ok(found_file)
}

/// Decide if a file's integrity is valid (according to a previously-created manifest).
///
/// A [`FoundFile`]'s [`FileIntegrity`] is considered valid if:
///     1. its relative path to the root of the inventoried directory matches.
///     2. its MD5 hashe matches.
fn assess_integrity(inventoried_file: &FoundFile, manifest_entry: &FoundFile) -> Result<FileIntegrity, anyhow::Error> {
    // todo: note that file audit is "in progress" (for GUI column).
    let md5_hash_matches = &manifest_entry.md5_hash == &inventoried_file.md5_hash;

    // Log: Note whether MD5 hashes match.
    match md5_hash_matches {
        true => trace!("MD5 hashes match"),
        false => trace!("MD5 hashes don't match")
    };

    let integrity_detail = FileIntegrityDetail {
        // We can safely assume that the file path has already been found.
        file_path_matches: true,
        md5_hash_matches,
    };

    // todo: Add SHA1 hashing.

    // Consider a file to have passed audit if the file path and MD5 hash match.
    let decided_file_integrity = match integrity_detail.file_path_matches && integrity_detail.md5_hash_matches {
        true => FileIntegrity::Verified(integrity_detail),
        false => FileIntegrity::VerificationFailed(integrity_detail),
    };

    debug!("Assessed integrity of manifest entry {manifest_entry:?} \
            and found it to be {decided_file_integrity:?}");
    Ok(decided_file_integrity)
}

/// Verification manifest from a previous run.
///
/// These are loaded from manifest files with [`load_previous_manifest`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationManifest {
    pub file_path: PathBuf,
    date_created: NaiveDateTime,
}

/// Load [`FoundFile`]s from a previously-created (CSV) manifest file.
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

    let mut manifest_entries: Vec<FoundFile> = vec![];
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

        manifest_entries.push(found_file);
    }

    let audit_entry_count = manifest_entries.len();
    info!("Loaded {audit_entry_count:?} manifest entries");
    Ok(manifest_entries)
}

#[cfg(test)]
mod tests{
    use super::audit_directory_inventory;

    use test_log::test;

    #[test_log::test]
    fn test_audit_directory_all_verified() {

    }
}
