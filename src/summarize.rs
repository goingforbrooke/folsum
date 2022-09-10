use std::collections::HashMap;
use std::ffi::OsString;
use std::ffi::OsStr;
use std::path::PathBuf;
use walkdir::WalkDir;


pub fn catalog_directory<'a>(target_dir: &PathBuf, extension_counts: &'a mut HashMap<String, i128>) -> &'a mut HashMap<String, i128> {
    // Reset file extension counts to zero.
    *extension_counts = HashMap::new();
    // Categorize all extensionless files as "No extension."
    let default_extension = OsString::from("No extension");
    // Recursively iterate through each subdirectory and don't add subdirectories to the result.
    for entry in WalkDir::new(target_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir()) {
        // Extract the file extension from the file's name.
        let file_ext: &OsStr = entry.path().extension().unwrap_or(&default_extension);
        let show_ext: String = String::from(file_ext.to_string_lossy());
        // Add newly encountered file extensions to known file extensions with a counter of 0.
        let counter: &mut i128 = extension_counts.entry(show_ext).or_insert(0);
        // Increment the counter for known file extensions by one.
        *counter += 1;
    }
    extension_counts 
}