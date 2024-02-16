#[cfg(test)]
pub mod test_utilities {
    pub struct TempEnvVar {
        variable_name: &'static str,
    }

    impl TempEnvVar {
        pub fn new(variable_name: &'static str, desired_value: &str) -> Self {
            std::env::set_var(variable_name, desired_value);
            Self { variable_name }
        }
    }

    impl Drop for TempEnvVar {
        fn drop(&mut self) {
            std::env::remove_var(self.variable_name);
        }
    }
}
