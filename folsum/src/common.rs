// Std crates for macOS, Windows, *and* WASM builds.
use std::path::PathBuf;

/// Add a debug-only `println!` macro
///
/// This ignores `--release`s, so stdout will only show in `cargo build` and `cargo run`.
/// todo: Remove `debug_println` from `common.rs`.
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}

/// Files found by FolSum.
#[derive(Debug, Default)]
pub struct FoundFile {
    pub file_path: PathBuf,
    pub md5_hash: u32,
}

#[cfg(test)]
pub mod test_utilities {
    use anyhow::{bail, Result};

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
