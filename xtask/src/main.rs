use std::{
    env,
    fs::{read_to_string, create_dir_all},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use::toml::Value;

use tauri_bundler::{SettingsBuilder, bundle_project, PackageSettings, BundleSettings, Settings, BundleBinary, Bundle};
use tauri_bundler::PackageType::{MacOsBundle};

fn main() {
    println!("Hello, world!");

    // Get the path to the `cargo` executable in a reliable way. Defaults to `cargo` if not found.
    let cargo_path: String = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    println!("cargo exe: {}", cargo_path);
    // Get the path to the `cargo` manifest directory in a reliable way.
    let cargo_manifest_dir: String = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo manifest dir: {}", cargo_manifest_dir);
    // Get the path to FolSum's root directory in a reliable way.
    let project_root: PathBuf = get_project_root();
    println!("project root: {:?}", project_root); 
    // Assume that FolSum's root directory is is the `folsum/folsum/` subdirectory.
    let folsum_root: PathBuf = project_root.join("folsum");
    println!("folsum root: {:?}", folsum_root); 

    // Build binaries so we can put them into a `.app` bundle.
    build(cargo_path, &project_root);
    
    // Bundle binaries.
    let bundle_paths = bundle(&folsum_root, &project_root);
    println!("bundle paths: {:?}", bundle_paths);
}

fn build(cargo_path: String, project_root: &PathBuf) {
    println!("cargo exe: {}", cargo_path);
    println!("project root: {:?}", project_root);
    // Run `cargo build --release` in `folsum/folsum/`.
    println!("Starting build with `cargo build --release`");
    let build_result: Output = Command::new(cargo_path)
        .current_dir(project_root)
        .args(&["build", "--release"])
        .output()
        .expect("Failed to cargo build FolSum");
    println!("build status: {}", build_result.status);
    println!("Cargo build output:");
    // Pass the build command's stdout and stderr through to the parent process.
    io::stdout().write_all(&build_result.stdout).unwrap();
    io::stderr().write_all(&build_result.stderr).unwrap();
    // Ensure that the build succeeded.
    assert!(build_result.status.success());
}

fn bundle(folsum_root: &PathBuf, project_root: &PathBuf) -> Result<Vec<Bundle>, Box<dyn std::error::Error>> {
    // Assume that FolSum's `Cargo.toml` is `folsum/folsum/Cargo.toml`.
    let folsum_cargo: PathBuf = folsum_root.join("Cargo.toml");
    println!("folsum cargo: {:?}", folsum_cargo);
    // Read Cargo.toml.
    let cargo_contents: String = read_to_string(folsum_cargo).expect("Failed to read Cargo.toml");
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
    let extracted_icon_dir: &str = cargo_values["package"]["metadata"]["bundle"]["icon"].as_str().expect("Failed to extract bundle icon directory");
    println!("Extracted bundle icon directory: {}", extracted_icon_dir);
    let icon_dir: PathBuf = folsum_root.join(extracted_icon_dir);
    println!("Bundle icon directory: {:?}", icon_dir);
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
    // Sort bundle icons for predictability so it's easier to troubleshoot bundler icon errors.
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
    let output_dir: PathBuf = folsum_root.join("target/release/");
    println!("Output directory: {:?}", output_dir);
    // Ensure that the output directory exists.
    create_dir_all(&output_dir).expect("Failed to create output directory for bundle");
    // Extract binary name.
    // todo: Use binary name from [[bin]] in Cargo.toml instead of assuming it's (the package_name) `folsum`.
    let binary_name: &str = cargo_values["package"]["name"].as_str().expect("Failed to extract binary name");
    // Expect the (universal) binary (created by `lipo`) to be `target/release/folsum`.
    let binary_path: PathBuf = output_dir.join(binary_name);

    // Temp: Override binary path with `cargo build --release` standard path (`folsum/target/release/folsum`).
    let binary_path: PathBuf = project_root.join("target/release/folsum");

    // Ensure that the binary exists and that it's a file. Otherwise, panic decriptively.
    match binary_path.is_file() {
        true => println!("Found binary at {:?}", binary_path),
        false => {
            if binary_path.exists() {
                panic!("Path to the binary {:?} exists, but is not a file", binary_path);
            } else {
                panic!("Path to the binary {:?} does not exist", binary_path);
            }
        }
    }
    // Create binary settings for Tauri Bundler. Use the package name as the binary name and mark it as thing to be executed.
    let binary_settings: BundleBinary = BundleBinary::new(package_name.to_string(), true)
        .set_src_path(Some(binary_path.into_os_string().into_string().unwrap()));
    println!("Defined binary settings");

    // Make a settings builder for Tauri Bundler.
    let settings_builder: SettingsBuilder = SettingsBuilder::new()
        // Add package settings to settings builder.
        .package_settings(package_settings)
        // Add bundle settings to settings builder.
        .bundle_settings(bundle_settings)
        // Add binary settings to the settings builder.
        .binaries(vec![binary_settings])
        // Set the project output directory.
        .project_out_directory(&output_dir)
        // Set the package type to MacOsBundle.
        .package_types(vec![MacOsBundle]);
    println!("Defined all bundler settings");

    let bundler_settings: Settings = settings_builder.build().expect("Failed to build bundler settings");
    println!("Built bundler settings");

    // Bundle the project.
    let completed_bundles: Vec<Bundle> = bundle_project(bundler_settings)?;
    println!("Bundled project");
    Ok(completed_bundles)
}

fn get_project_root() -> PathBuf {
    // Get the path to the project root, as defined by `Cargo.toml` in the project root (with the workspace members field).
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
