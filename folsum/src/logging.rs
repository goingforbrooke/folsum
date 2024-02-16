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
/// A logfile directory named `logs/` for the application is
/// created in a platform-specific app data directory. If it
/// already exists, then nothing happens.
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
    let lowercased_name = app_name.to_lowercase();
    // Define logs dir as `<app_name>/logs/` in app data dir.
    let log_dir = parent_dir.join(lowercased_name).join("logs");
    // Ensure that logs dir and its parents exist.
    // todo: Handle logdir creation errors.
    create_dir_all(&log_dir)?;
    //todo: Separate debug messages for logdir did/didn't already exist.
    Ok(log_dir)
}

/// Create a logfile in the loging subdirectory for this application.
///
/// Name the logfile `<app_name>.log`. If the logfile already exists, then nothing happens.
fn create_logfile(app_name: &str, parent_dir: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let lowercased_name = app_name.to_lowercase();
    // todo: Store logfiles in a subdir named after `name` in `[package]` of Cargo.toml.
    let logfile_name = format!("{}.log", lowercased_name);
    let logfile_path = parent_dir.join(PathBuf::from(logfile_name));
    // Ensure the logfile exists.
    File::create(&logfile_path)?;
    Ok(logfile_path)
}

/// Define how log records are diplayed in the log file.
fn define_logfile_format(logfile_path: &PathBuf) -> Result<fern::Dispatch, Box<dyn Error>> {
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
pub fn setup_native_logging(app_name: &str) -> Result<(), fern::InitError> {
    // todo: Provide for logdir creation failures.
    let logdir = create_logdir(&app_name, None).unwrap();
    // todo: Provide for logfile creation failures.
    let logfile_path = create_logfile(&app_name, &logdir).unwrap();
    let console_config = define_console_format();
    // MacOS: `~/Library/Application\ Support/folsum/logs/folsum.log`
    let file_config = define_logfile_format(&logfile_path);
    // Activate the console logger and the file logger.
    fern::Dispatch::new()
        .chain(console_config.unwrap())
        // todo: Provide for logging file config creation failures.
        .chain(file_config.unwrap())
        .apply()?;
    info!("Initialized logger with target file {logfile_path:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_utilities::TempEnvVar;

    use tempdir::TempDir;

    #[test]
    fn test_create_logdir() {
        // Create temporary directory that'll be deleted when it goes out of scope.
        let temp_dir = TempDir::new("example").unwrap();

        // Use the tempdir by manipulating `dirs` crate's use of `$HOME`.
        if cfg!(unix) {
            std::env::set_var("HOME", &temp_dir.path());
        } else if cfg!(windows) {
            std::env::set_var("USERPROFILE", temp_dir.path());
        }

        const TEST_APP_NAME: &str = "TestAppName";

        let platform_path = PathBuf::from(format!(
            // Exclude leading forward slash to prevent total replacement of `temp_dir`.
            "Library/Application Support/{}/logs/",
            TEST_APP_NAME.to_lowercase()
        ));
        let expected_logdir = temp_dir.path().join(platform_path);

        //let _ = setup_native_logging(&TEST_APP_NAME);
        let _ = create_logdir(&TEST_APP_NAME.to_lowercase(), None);

        assert!(expected_logdir.exists(), "Logging directory wasn't created");

        // Clean up env vars.
        if cfg!(unix) {
            std::env::remove_var("HOME");
        } else if cfg!(windows) {
            std::env::remove_var("USERPROFILE");
        }
    }
}
