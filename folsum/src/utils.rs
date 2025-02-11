// Std crates for macOS, Windows, *and* WASM builds.
use std::collections::HashMap;

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// Add `iter()` to HashMap for sorting.
use itertools::Itertools;

pub fn sort_counts(extension_counts: &HashMap<String, u32>) -> Vec<(&String, &u32)> {
    // Alphabetize file extensions before occurrence sorting so those with the same count appear alphabetically.
    let mut sorted_extensions: Vec<(&String, &u32)> = extension_counts.iter().sorted().collect();
    // Sort file extensions from most to least occurrences, assuming the user wants to see the most numerous filetypes first.
    sorted_extensions.sort_by(|a, b| b.1.cmp(a.1));
    trace!("Sorted extensions");
    sorted_extensions
}
