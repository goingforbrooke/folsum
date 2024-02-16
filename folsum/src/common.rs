#[cfg(test)]
pub mod test_utilities {
    use std::error::Error;

    pub struct TempEnvVar<'a> {
        variable_name: &'a str,
    }

    impl<'a> TempEnvVar<'a> {
        pub fn new(variable_name: &'a str, desired_value: &'a str) -> Self {
            std::env::set_var(variable_name, desired_value);
            Self { variable_name }
        }
    }

    impl<'a> Drop for TempEnvVar<'a> {
        fn drop(&mut self) {
            std::env::remove_var(self.variable_name);
        }
    }
}
