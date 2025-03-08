#![warn(clippy::all, rust_2018_idioms)]
// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::error::Error;

// External crates for WASM builds.
#[cfg(target_family = "wasm")]
use web_sys;
// Add `dyn_into` for getting HTML canvas in WASM builds.
use web_sys::wasm_bindgen::JsCast;

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
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(folsum::FolsumGui::new(cc)))),
            )
            .await;

        // Remove loading text and spinner.
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
