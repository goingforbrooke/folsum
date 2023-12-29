//! Logging
//!
//! `logging` sets up native logging for FolSum projects. This doesn't include WASM deployments,
//! which need a different logger.

// Standard library.
use std::error::Error;
use std::path::Path;
use std::time::SystemTime;

// External crates.
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, trace, warn};

/// Initialize a logger for native compilation targets.
///
/// # Examples
///
/// ```
/// trace!("doodle");
/// debug!("buuuuuuuuuuuugs!");
/// info!("knowledge");
/// warn!("uh-oh");
/// error!("danger will robinson");
/// Output:
/// 11:58🧊logging.rsL79::folsum::logging Initialized logger
/// 11:58🐛logging.rsL82::folsum::logging buuuuuuuuuuuugs!
/// 11:58🧊logging.rsL83::folsum::logging knowledge
/// 11:58💡logging.rsL84::folsum::logging uh-oh
/// 11:58🚨logging.rsL85::folsum::logging danger will robinson
/// ```
pub fn setup_native_logging() -> Result<(), Box<dyn Error>> {
    // Define the line color for each log level.
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .debug(Color::Green)
        .info(Color::Cyan)
        .trace(Color::White);
    // Create a foundation for the console logger and file logger to sit on top of.
    let base_config = fern::Dispatch::new();
    // Define how log records are displayed in the console.
    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{timestamp}{level_emoji}{record_filename}L{record_line}::{record_module} {color_line}{message}\x1B[0m",
                color_line = format_args!(
                    "\x1b[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                // Convert the log level to a fun emoji.
                level_emoji = match record.level() {
                    log::Level::Error => "🚨",
                    log::Level::Warn => "💡",
                    log::Level::Info => "🧊",
                    log::Level::Debug => "🐛",
                    log::Level::Trace => "🔎",
                },
                message = message,
                record_filename = record.file()
                    .and_then(|record_filepath| Path::new(record_filepath).file_name())
                    .and_then(|record_filename| record_filename.to_str())
                    .unwrap_or("unknown_file"),
                // Get the line number that the log record was invoked from.
                record_line = record.line().map_or(String::from("unknown_line"), |line| line.to_string()),
                record_module = record.module_path().unwrap_or("unknown_module"),
                timestamp = chrono::Local::now().format("%H:%M"),
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
                "[{timestamp} {record_filename}L{record_line}::{record_module}] {message}",
                timestamp = humantime::format_rfc3339_seconds(SystemTime::now()),
                // Get the full path to the invoking file.
                record_filename = record.file().unwrap_or("unknown_file"),
                // Get the line number that the log record was invoked from.
                record_line = record
                    .line()
                    .map_or(String::from("unknown_line"), |line| line.to_string()),
                record_module = record.module_path().unwrap_or("unknown_module"),
                message = message
            ));
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
