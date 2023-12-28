#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::{debug, error, info, trace, warn};
use std::error::Error;
use std::time::SystemTime;

use fern::colors::{Color, ColoredLevelConfig};

fn setup_native_logging() -> Result<(), Box<dyn Error>> {
    // Define the line color for each log level.
    let mut colors = ColoredLevelConfig::new().info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        // Set minimum logging level for all modules to DEBUG.
        .level(log::LevelFilter::Debug)
        // Send unfiltered messages to stdout.
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
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
