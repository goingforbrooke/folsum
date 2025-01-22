#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use log::info;
#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use folsum::setup_native_logging;

#[cfg(not(target_arch = "wasm32"))]
const APP_NAME: &str = "FolSum";

#[cfg(not(target_arch = "wasm32"))]
fn setup_native_eframe() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        &APP_NAME,
        native_options,
        Box::new(|cc| Box::new(folsum::FolsumGui::new(cc))),
    )?;
    info!("Initialized native eframe");
    Ok(())
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    setup_native_logging(&APP_NAME)?;
    setup_native_eframe()?;
    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
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
