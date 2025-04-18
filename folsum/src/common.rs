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
pub const FILEDATE_PREFIX_FORMAT: &str = "%Y-%-m-%-d-%-H-%-M";
pub const FOLSUM_CSV_EXTENSION: &str = ".folsum.csv";


/// Point in the process of inventorying a directory's contents.
#[derive(Clone)]
pub enum InventoryStatus {
    NotStarted,
    InProgress,
    Done,
}

/// Point in the process of creating a manifest export file.
#[derive(Clone, Debug)]
pub enum ManifestCreationStatus {
    NotStarted,
    InProgress,
    Done(PathBuf),
}

/// Integrity of the directory being inventoried.
#[derive(Clone, Debug)]
pub enum DirectoryAuditStatus {
    Unaudited,
    InProgress,
    Audited,
    DiscrepanciesFound,
}

/// Details about why a [`FoundFile`] succeeded or failed an audit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileIntegrityDetail {
    pub file_path_matches: bool,
    pub md5_hash_matches: bool,
}

/// Integrity of a file in a directory that's being inventoried.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum FileIntegrity {
    InProgress,
    #[default]
    Unverified,
    Verified(FileIntegrityDetail),
    VerificationFailed(FileIntegrityDetail),
    NewlyAdded,
}

/// Files found by FolSum.
#[derive(Clone, Debug, Default)]
pub struct FoundFile {
    // Relative path from the inventory directory to the file.
    pub file_path: PathBuf,
    // MD5 digest as a hexadecimal string.
    pub md5_hash: String,
    // Whether the file passed audit
    pub file_integrity: FileIntegrity,
}

impl FoundFile {
    pub fn new(file_path: PathBuf, md5_hash: String) -> Self {
        Self {
            file_path,
            md5_hash,
            file_integrity: FileIntegrity::default(),
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

        /// Get platform-specific environment variable that corresponds to `$HOME`.
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
