

#[cfg(test)]
pub mod test_utils {
    pub fn generate_mock_directory(test_dir: &PathBuf) -> std::io::Result<HashMap<String, u32>> {
        let mut current_path = test_dir.clone();
        // Create a test directory in the current directory.
        fs::create_dir(&current_path)?;
        // Define the file extensions that'll be used to create (empty) test files.
        let extensions = vec!["py", "pdf", "doc", "zip", "xml"];
        // Keep track of how many files of each extension are created.
        let mut extension_counts: HashMap<String, u32> = HashMap::new();
        // Create subdirectories with a depth of ten.
        for subdir_depth in 1..=10 {
            // Name each subdirectory for its depth.
            current_path.push(format!("subdir_{}", subdir_depth));
            fs::create_dir(&current_path)?;
            // Create a number of empty files in each subdirectory equal to the subdirectory depth.
            for counter in 1..subdir_depth {
                // Pick a file extension for this file based off of how deep the subdirectory is.
                let extension = &extensions[counter % extensions.len()];
                let filename = format!("file_{}.{}", counter, extension);
                let file_path = current_path.join(filename);
                // Create the empty file.
                fs::File::create(file_path)?;
                // If the file extension already exists, then add one to it, otherwise, add it with "zero."
                *extension_counts.entry(extension.to_string()).or_insert(0) += 1;
            }
        }
        // Return the number of files created for each extension as a test "answer key."
        Ok(extension_counts)
    }
}
