//! Logging
//!
//! `logging` sets up native logging for FolSum projects. This doesn't include WASM deployments,
//! which need a different logger.

// Standard library.
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// External crates.
use dirs::data_local_dir;
use fern::colors::{Color, ColoredLevelConfig};
#[allow(unused)]
use log::{debug, error, info, trace, warn};

/// Create application data subdirectory for logfiles.
///
/// A logfile directory for the application is created in a platform-specific
/// app data directory. If it already exists, then nothing happens.
fn create_logdir(
    app_name: &str,
    logdir_override: Option<&PathBuf>,
) -> Result<PathBuf, Box<dyn Error>> {
    // Get the place on the user's box where applications can store data.
    let appdata_dir = data_local_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find an app data directory to store log files",
        )
    })?;
    // Store logs the Appdata dir unless specified otherwise.
    let parent_dir = match logdir_override {
        Some(logdir_override) => logdir_override,
        None => &appdata_dir,
    };
    // Define logs dir as `<app_name>/logs/` in app data dir.
    let log_dir = parent_dir.join(app_name).join("logs");
    // Ensure that logs dir and its parents exist.
    // todo: Handle logdir creation errors.
    create_dir_all(&log_dir)?;
    Ok(log_dir)
}

/// Create a logfile in the loging subdirectory for this application.
///
/// Name the logfile `<app_name>.log`. If the logfile already exists, then nothing happens.
fn create_logfile(app_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    // todo: Store logfiles in a subdir named after `name` in `[package]` of Cargo.toml.
    let logfile_name = format!("{}.log", app_name);
    let logfile_path = PathBuf::from(logfile_name);
    // Ensure the logfile exists.
    File::create(&logfile_path)?;
    Ok(logfile_path)
}

/// Define how log records are diplayed in the log file.
fn define_logfile_format(logfile_path: &PathBuf) -> Result<fern::Dispatch, Box<dyn Error>> {
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
        // Append to a given logfile, creating it if necessary.
        .chain(fern::log_file(logfile_path)?);
    Ok(file_config)
}

fn define_console_format() -> Result<fern::Dispatch, Box<dyn Error>> {
    // Define the line color for each log level.
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .debug(Color::Green)
        .info(Color::Cyan)
        .trace(Color::White);
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
    Ok(stdout_config)
}

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
pub fn setup_native_logging() -> Result<(), fern::InitError> {
    let app_name = String::from("FolSum");

    let logdir = create_logdir(&app_name, None).unwrap();
    let logfile = create_logfile(&app_name).unwrap();
    let logfile_path = logdir.join(&logfile);

    let console_config = define_console_format();
    let file_config = define_logfile_format(&logfile_path);
    // Activate the console logger and the file logger.
    fern::Dispatch::new()
        .chain(console_config.unwrap())
        .chain(file_config.unwrap())
        .apply()?;
    info!("Initialized logger");
    Ok(())
}