use std::{
    env,
    fs::{create_dir_all, read_to_string},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use env_logger::Env;
use log::{debug, info};
use toml::Value;

use tauri_bundler::PackageType::MacOsBundle;
use tauri_bundler::{
    bundle_project, Bundle, BundleBinary, BundleSettings, PackageSettings, Settings,
    SettingsBuilder,
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    // Default to `info` log level, but allow the user to override via `RUST_LOG` environment variable.
    env_logger::Builder::from_env(Env::default().default_filter_or("xtask=info")).init();
    // If there was an error...
    if let Err(e) = try_main() {
        // ... then print it to stderr...
        debug!("{}", e);
        // ... and exit with a non-zero exit code.
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    // Get the path to FolSum's root directory in a reliable way.
    let project_root: PathBuf = get_project_root();
    debug!("project root: {:?}", project_root);
    // Assume that FolSum's root directory is is the `folsum/folsum/` subdirectory.
    let folsum_root: PathBuf = project_root.join("folsum");
    debug!("folsum root: {:?}", folsum_root);

    // Extract the first command line argument.
    let task: Option<String> = env::args().nth(1);
    match task.as_deref() {
        // If "build" was passed as the first command ine argument, then build the application.
        Some("build") => build(&project_root),
        // If "bundle" was passed as the first command line argument, then bundle the application.
        Some("bundle") => bundle(&folsum_root, &project_root),
        // If "dist" was passed as the first command line argument, then build and bundle the application.
        Some("dist") => dist(&folsum_root, &project_root),
        // If "help" was passed as the first command line argument, then describe available tasks.
        Some("help") => print_help(),
        // If the first command line argument was unrecognized, then describe available tasks.
        _ => print_help(),
    }
}

fn print_help() -> Result<(), DynError> {
    info!("Tasks:

           build           builds application
           dist            builds and bundles application (equivalent to running `build` and `bundle`)
           help            prints this help message
           "
    );
    Ok(())
}

fn dist(folsum_root: &PathBuf, project_root: &PathBuf) -> Result<(), DynError> {
    // Build binaries so we can put them into a `.app` bundle.
    build(&project_root)?;

    // Bundle binaries.
    bundle(&folsum_root, &project_root)?;
    info!("Bundled binaries into .app bundle");
    Ok(())
}

fn build(project_root: &PathBuf) -> Result<(), DynError> {
    // Get the path to the `cargo` executable in a reliable way. Defaults to `cargo` if not found.
    let cargo_path: String = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    debug!("using `cargo` executable: {}", cargo_path);
    // Run `cargo build --release` in `folsum/folsum/`.
    info!("Starting build with `cargo build --release`");
    let build_result: Output = Command::new(cargo_path)
        .current_dir(project_root)
        .args(&["build", "--release", "--color", "always"])
        .output()
        .expect("Failed to cargo build FolSum");
    debug!("build status: {}", build_result.status);
    info!("Cargo build output:");
    // Pass the build command's stdout and stderr through to the parent process.
    io::stdout().write_all(&build_result.stdout).unwrap();
    io::stderr().write_all(&build_result.stderr).unwrap();
    // Ensure that the build succeeded.
    assert!(build_result.status.success());
    Ok(())
}

fn bundle(folsum_root: &PathBuf, project_root: &PathBuf) -> Result<(), DynError> {
    // Assume that FolSum's `Cargo.toml` is `folsum/folsum/Cargo.toml`.
    let folsum_cargo: PathBuf = folsum_root.join("Cargo.toml");
    debug!("folsum cargo: {:?}", folsum_cargo);
    // Read Cargo.toml.
    let cargo_contents: String = read_to_string(folsum_cargo).expect("Failed to read Cargo.toml");
    // Parse Cargo.toml contents.
    let cargo_values: Value = cargo_contents
        .parse()
        .expect("Failed to parse Cargo contents");

    // Extract package name.
    let package_name: &str = cargo_values["package"]["name"]
        .as_str()
        .expect("Failed to extract package name");
    // Extract package version.
    let package_version: &str = cargo_values["package"]["version"]
        .as_str()
        .expect("Failed to extract package version");
    // Extract package description.
    let package_description: &str = cargo_values["package"]["description"]
        .as_str()
        .expect("Failed to extract package description");

    // Create package settings for Tauri Bundler with package values extracted from Cargo.toml.
    let package_settings: PackageSettings = PackageSettings {
        product_name: package_name.to_string(),
        version: package_version.to_string(),
        description: package_description.to_string(),
        homepage: None,
        authors: None,
        default_run: None,
    };

    // Extract bundle identifier.
    let bundle_identifier: &str = cargo_values["package"]["metadata"]["bundle"]["identifier"]
        .as_str()
        .expect("Failed to extract bundle identifier");

    // Extract bundle icons.
    let bundle_icons: Vec<PathBuf> = cargo_values["package"]["metadata"]["bundle"]["icon"]
        .as_array()
        .expect("Failed to extract bundle icon paths")
        .iter()
        .map(|icon_path| {
            folsum_root.join(
                icon_path
                    .as_str()
                    .expect("Failed to extract bundle icon path"),
            )
        })
        .collect();
    debug!("Found bundle icons:\n{:?}", bundle_icons);

    // Extract bundle copyright.
    let bundle_copyright: &str = cargo_values["package"]["metadata"]["bundle"]["copyright"]
        .as_str()
        .expect("Failed to extract bundle copyright");

    // Create bundle settings for Tauri Bundler with bundle values extracted from Cargo.toml.
    let bundle_settings: BundleSettings = BundleSettings {
        identifier: Some(bundle_identifier.to_string()),
        // Convert each bundle icon path to a string.
        icon: Some(
            bundle_icons
                .iter()
                .map(|icon_path: &PathBuf| icon_path.to_str().unwrap().to_string())
                .collect(),
        ),
        copyright: Some(bundle_copyright.to_string()),
        ..Default::default()
    };

    // Create bundles in (new directory)`target/release/bundle`.
    //let output_dir: PathBuf = folsum_root.join("target/release/");
    // Temp: Override output directory path with
    let output_dir: PathBuf = project_root.join("target/release/");
    debug!("Output directory: {:?}", output_dir);

    // Ensure that the output directory exists.
    create_dir_all(&output_dir).expect("Failed to create output directory for bundle");

    // Extract binary name.
    // todo: Use binary name from [[bin]] in Cargo.toml instead of assuming it's (the package_name) `folsum`.
    let binary_name: &str = cargo_values["package"]["name"]
        .as_str()
        .expect("Failed to extract binary name");
    // Expect the (universal) binary (created by `lipo`) to be `target/release/folsum`.
    let binary_path: PathBuf = output_dir.join(binary_name);

    // Ensure that the binary exists and that it's a file. Otherwise, panic decriptively.
    match binary_path.is_file() {
        true => debug!("Found binary at {:?}", binary_path),
        false => {
            if binary_path.exists() {
                panic!(
                    "Path to the binary {:?} exists, but is not a file",
                    binary_path
                );
            } else {
                panic!("Path to the binary {:?} does not exist", binary_path);
            }
        }
    }
    // Create binary settings for Tauri Bundler. Use the package name as the binary name and mark it as thing to be executed.
    let binary_settings: BundleBinary = BundleBinary::new(binary_name.to_string(), true)
        .set_src_path(Some(binary_path.into_os_string().into_string().unwrap()));
    debug!("Defined binary settings: {:?}", binary_settings);

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
    debug!("Defined all bundler settings");

    let bundler_settings: Settings = settings_builder
        .build()
        .expect("Failed to build bundler settings");
    debug!("Built bundler settings");

    // Bundle the project.
    let completed_bundles: Vec<Bundle> = bundle_project(bundler_settings)?;
    info!("Bundled project: {:?}", completed_bundles);
    Ok(())
}

fn get_project_root() -> PathBuf {
    // Get the path to the project root, as defined by `Cargo.toml` in the project root (with the workspace members field).
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
