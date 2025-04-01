// Std crates for macOS, Windows, *and* WASM builds.
use std::path::PathBuf;

/// Add a debug-only `println!` macro.
///
/// We use this in `logging.rs` to note if we encounter logging setup errors.
///
/// This ignores `--release`s, so stdout will only show in `cargo build` and `cargo run`.
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}

pub const CSV_HEADERS: &str = "File Path, MD5 Hash\n";
pub const FILEDATE_PREFIX_FORMAT: &str = "%-y-%-m-%-d-%-H-%-M";


// Point in the summarization process of a directory's contents.
#[derive(Clone)]
pub enum SummarizationStatus {
    NotStarted,
    InProgress,
    Done,
}


// Point in the process of creating a manifest export file.
#[derive(Clone)]
pub enum ManifestCreationStatus {
    NotStarted,
    InProgress,
    Done,
}


/// Integrity of the whole directory being summarized.
#[derive(Clone, Debug)]
pub enum DirectoryVerificationStatus {
    Unverified,
    InProgress,
    Verified,
    VerificationFailed,
}

/// Details about why a [`FoundFile`] succeeded or failed verification.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IntegrityDetail {
    pub file_path_matches: bool,
    pub md5_hash_matches: bool,
}

/// Integrity of a file in a directory that's being summarized.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum FileIntegrity {
    #[default]
    Unverified,
    InProgress,
    Verified(IntegrityDetail),
    VerificationFailed(IntegrityDetail),
}

/// Files found by FolSum.
#[derive(Clone, Debug, Default)]
pub struct FoundFile {
    // Relative path from the summarization directory to the file.
    pub file_path: PathBuf,
    // MD5 digest as a hexadecimal string.
    pub md5_hash: String,
    // Whether the file passed verification.
    pub file_verification_status: FileIntegrity,
}

impl FoundFile {
    pub fn new(file_path: PathBuf, md5_hash: String) -> Self {
        Self {
            file_path,
            md5_hash,
            file_verification_status: FileIntegrity::default(),
        }
    }
}

#[cfg(test)]
pub mod test_utilities {
    use anyhow::{bail, Result};

    /// Test utility for temporarily changing `$HOME` environment variable.
    ///
    /// Used for the setup of [`crate::logging::test_create_appdata_logdir`].
    pub struct TempHomeEnvVar {
        variable_name: String,
    }

    impl TempHomeEnvVar {
        pub fn new(desired_value: &str) -> Result<Self> {
            let variable_name = Self::get_platform_env_var()?;
            std::env::set_var(&variable_name, desired_value);
            debug_println!(
                "Added temporary env var: {}:{}",
                &variable_name,
                &desired_value
            );
            Ok(Self { variable_name })
        }

        /// Get platform-specifc environment variable that corresponds to `$HOME`.
        pub fn get_platform_env_var() -> Result<String> {
            let platform = if cfg!(unix) {
                "unix"
            } else if cfg!(windows) {
                "windows"
            } else {
                "unknown"
            };
            let env_var_name = match platform {
                "unix" => "HOME",
                "windows" => "USERPROFILE",
                // todo: Raise more specific test util (?Anyhow?) setup error for unknown platform.
                _ => bail!("Unsupported platform"),
            };
            Ok(env_var_name.to_string())
        }
    }

    impl Drop for TempHomeEnvVar {
        fn drop(&mut self) {
            std::env::remove_var(&self.variable_name);
            debug_println!(
                "Automatically removed temporary env var: {}",
                self.variable_name
            );
        }
    }
}
