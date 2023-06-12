use tauri_bundler::{SettingsBuilder, bundle_project, PackageSettings};

fn main() {
    //let mut settings_builder = SettingsBuilder::new()
    //  .package_settings(PackageSettings)
    //  .bundle_settings(self.get_bundle_settings(config, &enabled_features)?)
    //  .binaries(self.get_binaries(config, &target)?)
    //  .project_out_directory(out_dir)
    //  .target(target);
    let package_settings = 

    // Extract package settings from Cargo.toml and bundle settings from Cargo.toml's [package.metadata.tauri.bundle].
    let bundle_settings = SettingsBuilder::new().build();

    println!("{:?}", bundle_settings);

    // Bundle the project.
    //if let Err(e) = bundle_project(bundle_settings.unwrap()) {
    //    panic!("Failed to bundle application: {}", e);
    //}
}
