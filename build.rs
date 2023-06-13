use std::fs::{read_to_string, create_dir_all};
use std::path::Path;

use::toml::Value;

use tauri_bundler::{SettingsBuilder, bundle_project, PackageSettings, BundleSettings, Settings};
use tauri_bundler::PackageType::{MacOsBundle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read Cargo.toml.
    let cargo_contents: String = read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    // Parse Cargo.toml contents.
    let cargo_values: Value = cargo_contents.parse().expect("Failed to parse Cargo contents");

    // Extract package name.
    let package_name: &str = cargo_values["package"]["name"].as_str().expect("Failed to extract package name");
    // Extract package version.
    let package_version: &str = cargo_values["package"]["version"].as_str().expect("Failed to extract package version");
    // Extract package description.
    let package_description: &str = cargo_values["package"]["description"].as_str().expect("Failed to extract package description");

    // Create package settings for Tauri Bundler with package values extracted from Cargo.toml.
    let package_settings: PackageSettings = PackageSettings{
        product_name: package_name.to_string(),
        version: package_version.to_string(),
        description: package_description.to_string(),
        homepage: None,
        authors: None,
        default_run: None,
    };

    // Extract bundle identifier.
    let bundle_identifier: &str = cargo_values["package"]["metadata"]["bundle"]["identifier"].as_str().expect("Failed to extract bundle identifier");

    // Extract bundle icon directory.
    let icon_dir: &str = cargo_values["package"]["metadata"]["bundle"]["icon"].as_str().expect("Failed to extract bundle icon directory");
    println!("Bundle icon directory: {}", icon_dir);
    // Get every PNG file in the bundle icon directory.
    let mut bundle_icons: Vec<String> = std::fs::read_dir(icon_dir).expect("Failed to read bundle icon directory").map(|entry| {
        let dir_item: std::fs::DirEntry = entry.expect("Failed to read bundle icon directory entry");
        let path: std::path::PathBuf = dir_item.path();
        let path: &str = path.to_str().expect("Failed to convert bundle icon path to string");
        path.to_string()
    // Select only PNG files as icons.
    }).filter(|path| {
        path.ends_with(".png")
    }).collect();
    // If no bundle icons were found, then raise an error.
    if bundle_icons.is_empty() {
        panic!("No bundle icons found in bundle icon directory");
    }
    // Sort bundle icons.
    bundle_icons.sort();
    println!("Found bundle icons:\n {}", bundle_icons.join(", \n"));

    // Extract bundle copyright.
    let bundle_copyright: &str = cargo_values["package"]["metadata"]["bundle"]["copyright"].as_str().expect("Failed to extract bundle copyright");

    // Create bundle settings for Tauri Bundler with bundle values extracted from Cargo.toml.
    let bundle_settings: BundleSettings = BundleSettings {
        identifier: Some(bundle_identifier.to_string()),
        icon: Some(bundle_icons),
        copyright: Some(bundle_copyright.to_string()),
        ..Default::default()
    };

    // Create bundles in (new directory)`target/release/bundle`.
    let output_dir: &Path = Path::new("target/release/");
    // Ensure that output directory exists.
    create_dir_all(output_dir).expect("Failed to create output directory");

    // Make a settings builder for Tauri Bundler.
    let settings_builder: SettingsBuilder = SettingsBuilder::new()
        // Add package settings to settings builder.
        .package_settings(package_settings)
        // Add bundle settings to settings builder.
        .bundle_settings(bundle_settings)
        // Set the project output directory.
        .project_out_directory(output_dir)
        // Set the package type to MacOsBundle.
        .package_types(vec![MacOsBundle]);

    let bundler_settings: Settings = settings_builder.build().expect("Failed to build settings");

    //let mut settings_builder = SettingsBuilder::new()
    //  .package_settings(PackageSettings)
    //  .bundle_settings(self.get_bundle_settings(config, &enabled_features)?)
    //  .binaries(self.get_binaries(config, &target)?)
    //  .project_out_directory(out_dir)
    //  .target(target);

    // Bundle the project.
    let completed_bundles = bundle_project(bundler_settings);
    completed_bundles?;
    Ok(())
}
