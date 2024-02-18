/// Add a debug-only `println!` macro
///
/// This ignores `--release`s, so stdout will only show in `cargo build` and `cargo run`.
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}

#[cfg(test)]
pub mod test_utilities {
    use std::error::Error;

    pub struct TempEnvVar<'a> {
        variable_name: &'a str,
    }

    impl<'a> TempEnvVar<'a> {
        pub fn new(variable_name: &'a str, desired_value: &'a str) -> Self {
            std::env::set_var(variable_name, desired_value);
            debug_println!("Added env var: {}:{}", variable_name, desired_value);
            Self { variable_name }
        }
    }

    impl<'a> Drop for TempEnvVar<'a> {
        fn drop(&mut self) {
            std::env::remove_var(self.variable_name);
            debug_println!("Removed env var: {}", self.variable_name);
        }
    }

    /// Get platform-specifc environment variable that corresponds to `$HOME`.
    pub fn get_platform_env_var() -> Result<String, Box<dyn Error>> {
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
            _ => "unknown",
        };
        if env_var_name == "unknown" {
            // todo: Raise more specific test util (?Anyhow?) setup error for unknown platform.
            panic!("Unknown platform")
        }
        Ok(String::from(env_var_name))
    }
}
