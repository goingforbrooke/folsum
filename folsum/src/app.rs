use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{DateTime, Local};
use dirs::home_dir;
use egui_extras::{TableBuilder, Column};
use itertools::Itertools;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use walkdir::WalkDir;
use web_time::{Duration, Instant, SystemTime};

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Define default fields when deserializing old state.
pub struct TemplateApp {
    // Unique file extensions and the number of times each one was encountered.
    #[serde(skip)]
    extension_counts: Arc<Mutex<HashMap<String, u32>>>,
    // Number of files summarized, which doesn't include files and directories that were skipped.
    #[serde(skip)]
    total_files: u32,
    // User's chosen directory that will be recursively summarized when the "Summarize" button's clicked.
    #[serde(skip)]
    summarization_path: Arc<Mutex<Option<PathBuf>>>,
    // User's chosen directory and filename for CSV exports.
    export_file: Arc<Mutex<Option<PathBuf>>>,
    // Time that summarization starts so it can be used to calculate the time taken.
    #[serde(skip)]
    summarization_start: Arc<Mutex<Instant>>,
    // Amount of time that it takes to summarize a directory.
    #[serde(skip)]
    time_taken: Arc<Mutex<Duration>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            extension_counts: Arc::new(Mutex::new(HashMap::new())),
            total_files: 0,
            summarization_path: Arc::new(Mutex::new(None)),
            export_file: Arc::new(Mutex::new(None)),
            summarization_start: Arc::new(Mutex::new(Instant::now())),
            time_taken: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }
}

impl TemplateApp {
    // Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customized the look at feel of egui using `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            // You must enable the `persistence` feature for this to work.
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    // Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            extension_counts,
            total_files,
            summarization_path,
            export_file,
            summarization_start,
            time_taken,
            ..
        } = self;

        // Update the count of total files summarized.
        *total_files = extension_counts.lock().unwrap().values().sum();
        // Update the screen on each iteration, bounded by the refresh rate of the user's screen.
        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Add a menu bar to the top of the screen.
            egui::menu::bar(ui, |ui| {
                // Don't include a File->Quit menu item when compiling for web.
                #[cfg(not(target_arch = "wasm32"))]
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                // Add a dark/light mode toggle button to the top menu bar.
                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Choose a Directory to Summarize");

                // Don't add a directory picker when compiling for web.
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Open directory...").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        *summarization_path = Arc::new(Mutex::new(Some(path)));
                    }
                }

                ui.horizontal(|ui| {
                    let unlocked_path: &Option<PathBuf> = &*summarization_path.lock().unwrap();
                    // Check if the user has picked a directory to summarize.
                    let shown_path: &str = match &*unlocked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No directory selected",
                    };
                    ui.label("Chosen directory:");
                    // Display the user's chosen directory in monospace font.
                    ui.monospace(shown_path);
                });

                ui.separator();

                if ui.button("Summarize").clicked() {
                    let unlocked_path: &mut Option<PathBuf> = &mut *summarization_path.lock().unwrap();
                    // If the user picked a directory to summarize....
                    if unlocked_path.is_some() {
                        // ...then recursively count file extensions in the chosen directory.
                        // Reset file extension counts to zero.
                        *extension_counts.lock().unwrap() = HashMap::new();

                        // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                        let extension_counts_copy = Arc::clone(&extension_counts);
                        let summarization_path_copy = Arc::clone(&summarization_path);
                        let start_copy = Arc::clone(&summarization_start);
                        let time_taken_copy = Arc::clone(&time_taken);

                        thread::spawn(move || {
                            // Categorize extensionless files as "No extension."
                            let default_extension = OsString::from("No extension");

                            // Start the stopwatch for summarization time.
                            let mut unlocked_start_copy = start_copy.lock().unwrap();
                            *unlocked_start_copy = Instant::now();

                            let unlocked_summarization_path = summarization_path_copy.lock().unwrap();
                            // Clone the user's chosen path so we can release it's lock, allowing live table updates.
                            let summarization_path_copy = unlocked_summarization_path.clone();
                            // Release the mutex lock on the chosen path so extension count table can update.
                            drop(unlocked_summarization_path);

                            // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                            for entry in WalkDir::new(summarization_path_copy.unwrap())
                                .min_depth(1)
                                .into_iter()
                                .filter_map(Result::ok)
                                .filter(|e| !e.file_type().is_dir())
                            {
                                // Extract the file extension from the file's name.
                                let file_ext: &OsStr =
                                    entry.path().extension().unwrap_or(&default_extension);
                                let show_ext: String = String::from(file_ext.to_string_lossy());
                                // Lock the extension counts variable so we can add a file to it.
                                let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
                                // Add newly encountered file extensions to known file extensions with a counter of 0.
                                let counter: &mut u32 =
                                    unlocked_counts_copy.entry(show_ext).or_insert(0);
                                // Increment the counter for known file extensions by one.
                                *counter += 1;
                                // Update the summarization time stopwatch.
                                let mut unlocked_time_taken_copy = time_taken_copy.lock().unwrap();
                                *unlocked_time_taken_copy = unlocked_start_copy.elapsed();
                            }
                        });
                    };
                };

                ui.horizontal(|ui| {
                    let unlocked_time_taken = time_taken.lock().unwrap();
                    ui.label(format!(
                        "Summarized {} files in {} milliseconds",
                        &total_files,
                        &unlocked_time_taken.as_millis()
                    ));
                });

                ui.separator();

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Export to CSV").clicked() {
                    let date_today: DateTime<Local> = DateTime::from(SystemTime::now());
                    let formatted_date = date_today.format("%y_%m_%d").to_string();
                    // Prepend the date (YY_MM_DD) to the filename.
                    let export_filename = format!("{formatted_date}_folsum_export");
                    // Open the "Save export file as" dialog.
                    let starting_directory = match export_file.lock().unwrap().clone() {
                        // Open the export dialog in the same dir as the previous export.
                        Some(export_file) => export_file.parent().unwrap().to_path_buf(),
                        // Otherwise, if there was no previous export, then open the export dialog in the user's home dir.
                        None => home_dir().expect("Failed to get user's home directory")
                    };
                    // Ask user where they'd like to save the CSV export and what they'd like it to be called.
                    if let Some(path) = FileDialog::new()
                        // Add `.csv` to the end of the user's chosen name for the CSV export.
                        .add_filter("csv", &["csv"])
                        .set_title("Export extension counts to CSV file")
                        // Open export dialogs in the last saved directory (if it exists), otherwise in the user's home directory.
                        .set_directory(starting_directory)
                        // Set the default filename for CSV exports to YY_MM_DD_folsum_export.
                        .set_file_name(&export_filename)
                        .save_file() {
                        *export_file = Arc::new(Mutex::new(Some(path)));
                    }
                    // Copy extension counts so we can access them in a separate thread that's dedicated to this CSV dump.
                    let extension_counts_copy: Arc<Mutex<HashMap<String, u32>>> = Arc::clone(&extension_counts);
                    // Copy the export file path's `Arc` so we can access it in a separate thread for CSV dumping.
                    let export_file: Arc<Mutex<Option<PathBuf>>> = Arc::clone(&export_file);
                    thread::spawn(move || {
                        // Make a place to put extension counts that'll be written to the CSV file and include column headers.
                        let mut csv_rows = String::from("File Extension, Occurrences\n");
                        // Lock the extension counts so we can read them into CSV format.
                        let unlocked_extension_counts = extension_counts_copy.lock().unwrap();
                        for (extension_type, extension_count) in unlocked_extension_counts.iter() {
                            // Ensure that there are no commas or newlines in this extension's name that would disrupt the output format.
                            assert!(!extension_type.contains('\n') && !extension_type.contains(','));
                            let csv_row = format!("{extension_type},{extension_count}\n");
                            csv_rows.push_str(&csv_row)
                        }
                        // Lock the export file path so we can use it to create the CSV dump.
                        let export_file = export_file.lock().unwrap();
                        // Clone user's chosen export path so we can release it's lock, allowing live table updates.
                        let export_file = export_file.clone().unwrap();
                        // Create a CSV file to write the extension types and their counts to, overwriting it if it already exists.
                        let mut csv_export = File::create(export_file).expect("Failed to create CSV export file");
                        // Write the CSV's content to the export file.
                        csv_export.write_all(csv_rows.as_bytes()).expect("Failed to write contents to CSV export file")
                    });
                };

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("written with love by ");
                        ui.hyperlink_to("Brooke", "https://github.com/goingforbrooke");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Summarization by File Extension");
                ui.separator();
            });
            let unlocked_exts = extension_counts.lock().unwrap();
            // Alphabetize file extensions before occurrence sorting so those with the same count appear alphabetically.
            let mut ext_info: Vec<(&String, &u32)> = unlocked_exts.iter().sorted().collect();
            // Sort file extensions from most to least occurrences, assuming the user wants to see the most numerous filetypes first.
            ext_info.sort_by(|a, b| b.1.cmp(a.1));
            // todo: Optimize table by efficiently displaying viewable rows with `show_rows()`.
            // Create a scrollable table that (inefficiently) shows all rows, whether they're in the "viewport" or not.
            TableBuilder::new(ui)
                .resizable(true)
                .striped(true)
                .column(Column::initial(150.0).at_least(150.0))
                .column(Column::remainder().at_least(60.0))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("File Extension");
                    });
                    header.col(|ui| {
                        ui.heading("Occurrences");
                    });
                })
                .body(|mut body| {
                    for (extension_name, times_seen) in ext_info.iter() {
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label(extension_name.to_string());
                            });
                            row.col(|ui| {
                                ui.label(times_seen.to_string());
                            });
                        });
                    }
                });
        });
    }
}