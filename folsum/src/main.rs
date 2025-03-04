#![warn(clippy::all, rust_2018_idioms)]
// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::error::Error;

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// Internal crates for macOS, Windows, *and* WASM builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use folsum::setup_native_logging;

// The app name is only necessary for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
const APP_NAME: &str = "FolSum";

#[cfg(any(target_family = "unix", target_family = "windows"))]
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

// When compiling natively:
#[cfg(any(target_family = "unix", target_family = "windows"))]
fn main() -> Result<(), Box<dyn Error>> {
    setup_native_logging(&APP_NAME)?;
    setup_native_eframe()?;
    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_family = "wasm")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(folsum::FolsumGui::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
