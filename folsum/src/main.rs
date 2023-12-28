#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::info;
use std::error::Error;

fn setup_native_logging() -> Result<(), Box<dyn Error>> {
    // Define how messages should be logged.
    let logger_env = env_logger::Env::default()
        // Obviate defining `RUST_LOG` env var with `cargo run` by advancing log level from (default) ERROR to INFO.
        .filter_or("RUST_LOG", "INFO");
    env_logger::init_from_env(logger_env);
    info!("Initialized logger");
    Ok(())
}

fn setup_native_eframe() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "FolSum",
        native_options,
        Box::new(|cc| Box::new(folsum::FolsumGui::new(cc))),
    )?;
    info!("Initialized native eframe");
    Ok(())
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    setup_native_logging()?;
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
