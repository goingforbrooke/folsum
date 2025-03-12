use std::fs;
use std::path::PathBuf;

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};
use md5::compute as compute_md5;

/// Get the MD5 hash of a file.
pub fn get_md5_hash(file_path: &PathBuf) -> Result<String, anyhow::Error> {
    let loaded_bytes = fs::read(file_path)?;
    let computed_digest = compute_md5(&loaded_bytes);
    // Format MD5 digest as a lowercase hexadecimal string.
    let display_hash = format!("{computed_digest:x}");

    debug!("Computed MD5 hash {display_hash:?} for {file_path:?}");

    Ok(display_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_md5_hash() -> Result<(), anyhow::Error> {
        let content = b"Hello, world!";
        let expected_md5_hash = "6cd3556deb0da54bca060b4c39479839";

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content)?;
        temp_file.flush()?;

        let testfile_path = temp_file.path().to_path_buf();
        let actual_md5_hash = get_md5_hash(&testfile_path)?;

        assert_eq!(actual_md5_hash, expected_md5_hash);
        Ok(())
    }
}

