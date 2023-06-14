use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use egui_extras::{Size, TableBuilder, Column};
use itertools::Itertools;
use rfd::FileDialog;
use walkdir::WalkDir;

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Define default fields when deserializing old state.
pub struct TemplateApp {
    // Opt-out of member serialization with `#[serde(skip)]`.
    #[serde(skip)]
    // Unique file extensions and the number of times each one was encountered.
    extension_counts: Arc<Mutex<HashMap<String, u32>>>,
    #[serde(skip)]
    // Number of files summarized, which doesn't include files and directories that were skipped.
    total_files: u32,
    #[serde(skip)]
    // User's chosen directory that will be recursively summarized when the "Summarize" button's clicked.
    picked_path: Arc<Mutex<Option<PathBuf>>>,
    #[serde(skip)]
    // Note the time when summarization starts so it can be used to calculate the time taken.
    summarization_start: Arc<Mutex<Instant>>,
    #[serde(skip)]
    time_taken: Arc<Mutex<Duration>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            extension_counts: Arc::new(Mutex::new(HashMap::new())),
            total_files: 0,
            picked_path: Arc::new(Mutex::new(None)),
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
            picked_path,
            time_taken,
            summarization_start,
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
                        *picked_path = Arc::new(Mutex::new(Some(path)));
                    }
                }

                ui.horizontal(|ui| {
                    let unlocked_path: &Option<PathBuf> = &*picked_path.lock().unwrap();
                    // Check if the user has picked a directory to summarize.
                    let shown_path: &str = match &*unlocked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No directory selected",
                    };
                    ui.label("Chosen directory:");
                    // Display the user's chosen directory in monospace font.
                    ui.monospace(shown_path);
                });

                if ui.button("Summarize").clicked() {
                    let unlocked_path: &mut Option<PathBuf> = &mut *picked_path.lock().unwrap();
                    // If the user picked a directory to summarize....
                    if unlocked_path.is_some() {
                        // ...then recursively count file extensions in the chosen directory.
                        // Reset file extension counts to zero.
                        *extension_counts.lock().unwrap() = HashMap::new();

                        // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                        let extension_counts_copy = Arc::clone(&extension_counts);
                        let picked_path_copy = Arc::clone(&picked_path);
                        let start_copy = Arc::clone(&summarization_start);
                        let time_taken_copy = Arc::clone(&time_taken);

                        thread::spawn(move || {
                            // Categorize extensionless files as "No extension."
                            let default_extension = OsString::from("No extension");

                            // Start the stopwatch for summarization time.
                            let mut unlocked_start_copy = start_copy.lock().unwrap();
                            *unlocked_start_copy = Instant::now();

                            let unlocked_picked_path = picked_path_copy.lock().unwrap();
                            // Clone the user's chosen path so we can release it's lock, allowing live table updates.
                            let picked_path_copy = unlocked_picked_path.clone();
                            // Release the mutex lock on the chosen path so extension count table can update.
                            drop(unlocked_picked_path);

                            // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                            for entry in WalkDir::new(picked_path_copy.unwrap())
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
