//! Logging
//!
//! `logging` sets up native logging for FolSum projects. This doesn't include WASM deployments,
//! which need a different logger.

use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{bail, Result};
use dirs::data_local_dir;
use fern::colors::{Color, ColoredLevelConfig};

#[allow(unused)]
use log::{debug, error, info, trace, warn};

use crate::debug_println;

/// Create application data subdirectory for logfiles.
///
/// A logfile directory named `logs/` for the application is
/// created in a platform-specific app data directory. If it
/// already exists, then nothing happens.
fn create_appdata_logdir(app_name: &str) -> Result<PathBuf> {
    // Get the place on the user's box where applications can store data.
    let appdata_dir = match data_local_dir() {
        Some(appdata_dir) => appdata_dir,
        None => bail!("Failed to find an app data directory to store log files"),
    };
    let lowercased_name = app_name.to_lowercase();
    // Define logs dir as `<app_name>/logs/` in app data dir.
    let log_dir = appdata_dir.join(lowercased_name).join("logs");
    // Ensure that logs dir and its parents exist.
    create_dir_all(&log_dir)?;
    debug_println!("Created log dir: {:?}", log_dir);
    Ok(log_dir)
}

/// Create a logfile in the loging subdirectory for this application.
///
/// Name the logfile `<app_name>.log`. If the logfile already exists, then nothing happens.
fn create_logfile(app_name: &str, parent_dir: &PathBuf) -> Result<PathBuf> {
    let lowercased_name = app_name.to_lowercase();
    let logfile_name = format!("{}.log", lowercased_name);
    let logfile_path = parent_dir.join(PathBuf::from(logfile_name));
    // Ensure the logfile exists.
    File::create(&logfile_path)?;
    Ok(logfile_path)
}

/// Define how log records are displayed in the log file.
fn define_logfile_format(logfile_path: &PathBuf) -> Result<fern::Dispatch> {
    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{timestamp}_{record_filename}::{record_module}L{record_line}]_{message}",
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

/// Define how log lines should look in console output.
fn define_console_format() -> Result<fern::Dispatch> {
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
                "{timestamp}{level_emoji}{record_filename}::{record_module}L{record_line} {color_line}{message}\x1B[0m",
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
/// Simplified logs are sent to stdout, colorized by severity level. More complete logs are written
/// to a logfile in the user's local data directory.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// # use crate::folsum::setup_native_logging;
/// setup_native_logging("TestAppName");
/// use log::{debug, error, info, trace, warn};
/// //Output: 11:58🧊logging.rsL79::testappname::logging Initialized logger
/// trace!("doodle");
/// debug!("buuuuuuuuuuuugs!");
/// //Output: 11:58🐛logging.rsL82::testappname::logging buuuuuuuuuuuugs!
/// info!("knowledge");
/// //Output: 11:58🧊logging.rsL83::testappname::logging knowledge
/// warn!("uh-oh");
/// //Output: 11:58💡logging.rsL84::testappname::logging uh-oh
/// error!("danger will robinson");
/// //Output: 11:58🚨logging.rsL85::testappname::logging danger will robinson
/// ```
pub fn setup_native_logging(app_name: &str) -> Result<()> {
    let logdir = create_appdata_logdir(&app_name).unwrap();
    let logfile_path = create_logfile(&app_name, &logdir).unwrap();
    let console_config = define_console_format();
    let file_config = define_logfile_format(&logfile_path);
    // Activate the console logger and the file logger.
    fern::Dispatch::new()
        .chain(console_config.unwrap())
        .chain(file_config.unwrap())
        .apply()?;
    info!("Initialized logger with target file {logfile_path:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_utilities::TempHomeEnvVar;

    use tempfile::tempdir;

    #[test]
    fn test_create_appdata_logdir() {
        // Create temporary directory that'll be deleted when it goes out of scope.
        let temp_dir = tempdir().unwrap();

        // Use the tempdir by manipulating `dirs` crate's use of `$HOME`.
        // Set testing environment variable that'll be removed when this goes out of scope.
        let _temp_env_var = TempHomeEnvVar::new(&temp_dir.path().to_str().unwrap());

        // The application's name should be converted to lowercase.
        const TEST_APP_NAME: &str = "TestAppName";

        let platform_path = PathBuf::from(format!(
            // Exclude leading forward slash to prevent replacement of `temp_dir` in `.join()`.
            "Library/Application Support/{}/logs/",
            TEST_APP_NAME
        ));
        let expected_logdir = temp_dir.path().join(platform_path);

        debug_println!("$HOME: {:?}", std::env::var("HOME"));
        let _ = create_appdata_logdir(&TEST_APP_NAME);

        assert!(expected_logdir.exists(), "Logging directory wasn't created");
    }
}
