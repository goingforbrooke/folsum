#![warn(clippy::all, rust_2018_idioms)]
// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// Internal crates for macOS, Windows, *and* WASM builds.
use folsum::setup_native_logging;

// The app name is only necessary for macOS and Windows builds.
const APP_NAME: &str = "FolSum";

fn setup_native_eframe() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        &APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(folsum::FolsumGui::new(cc)))),
    )?;
    info!("Initialized native eframe");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_native_logging(&APP_NAME)?;
    setup_native_eframe()?;
    Ok(())
}
