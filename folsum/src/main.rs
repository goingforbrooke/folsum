#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::{debug, error, info, warn};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Define how messages should be logged.
    let logger_env = env_logger::Env::default();
    env_logger::init_from_env(logger_env);

    // Don't show egui log messages because FolSum's debugs gets lost in the sea of egui debugs.
    //logger_builder.filter_module("egui", LevelFilter::Info);

    info!("Initialized logger");
    debug!("Initialized logger");
    error!("Initialized logger");
    info!("Initialized logger");
    warn!("Initialized logger");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "FolSum",
        native_options,
        Box::new(|cc| Box::new(folsum::FolsumGui::new(cc))),
    )
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
