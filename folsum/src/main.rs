#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::{debug, error, info, trace, warn};
use std::error::Error;
use std::time::SystemTime;

use fern::colors::{Color, ColoredLevelConfig};

fn setup_native_logging() -> Result<(), Box<dyn Error>> {
    //struct AColor {
    //    r: u8,
    //    g: u8,
    //    b: u8,
    //}
    // Define what Grey looks like because it's not a color in the `colored`, used by `fern`.
    //enum MyColors {
    //    Blue,
    //    Grey {r: 51, g: 51, b: 51},
    //    Red,
    //    Yellow,
    //}
    // Define the line color for each log level.
    let colors_line = ColoredLevelConfig::new()
        .trace(Color::White)
        .debug(Color::White)
        .info(Color::Blue)
        .warn(Color::Yellow)
        .error(Color::Red);
    // Create a foundation for the console logger and file logger to sit on top of.
    let base_config = fern::Dispatch::new();
    // Define how log records are displayed in the console.
    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{date} {color_line} {level} {record_filename}::{record_module}] {color_line} {message}\x1B[0m",
                color_line = format_args!(
                    "\x1b[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = humantime::format_rfc3339_seconds(SystemTime::now()),
                // Colorize the log record based off of its log level.
                // todo: Add filename and line number.
                // Get the filename that the log record came from.
                record_filename = record.file().unwrap_or("unknown_file"),
                // Get the module that the log record came from.
                record_module = record.module_path().unwrap_or("unknown_module"),
                level = colors_line.color(record.level()),
                message = message,
            ));
        })
        // Ignore all non-warning GUI logs in console.
        .level_for("eframe", log::LevelFilter::Warn)
        .level_for("egui_glow", log::LevelFilter::Warn)
        .level_for("egui_winit", log::LevelFilter::Warn)
        // Console log remaining records at DEBUG and above.
        .level(log::LevelFilter::Debug)
        // Send unfiltered messages to stdout.
        .chain(std::io::stdout());
    // Define how log records are diplayed in the log file.
    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.target(),
                message
            ))
        })
        // Include logs records at every level.
        .level(log::LevelFilter::Trace)
        // Write to a file called `output.log` in the current working directory.
        .chain(fern::log_file("output.log")?);
    // Activate the console logger and the file logger.
    base_config
        .chain(stdout_config)
        .chain(file_config)
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
