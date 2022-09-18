use itertools::Itertools;
use rfd::FileDialog;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::ffi::OsString;
use std::ffi::OsStr;
use walkdir::WalkDir;
use std::sync::{Arc, Mutex};
use std::thread;

use egui_extras::{Size, TableBuilder};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    extension_counts: Arc<Mutex<HashMap<String, i128>>>,
    #[serde(skip)]
    total_files: i128,
    #[serde(skip)]
    picked_path: Arc<Mutex<Option<PathBuf>>>,
    #[serde(skip)]
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
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            extension_counts,
            total_files,
            picked_path,
            time_taken,
            summarization_start,
            ..
        } = self;

        // Show a live update of how many files have been summarized.
        *total_files = extension_counts.lock().unwrap().values().sum();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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
                    let unlocked_path: & Option<PathBuf> = & *picked_path.lock().unwrap();
                    //unlocked_path.un
                    // Check if the user has picked a directory to summarize.
                    let shown_path: &str = match & *unlocked_path {
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
                        let counts_copy = Arc::clone(&extension_counts);
                        let dir_copy = Arc::clone(&picked_path);
                        let start_copy = Arc::clone(&summarization_start);
                        let time_taken_copy = Arc::clone(&time_taken);
                        thread::spawn(move || {
                            // Categorize all extensionless files as "No extension."
                            let default_extension = OsString::from("No extension");
                            // Start the stopwatch for summarization time.
                            let mut unlocked_start_copy = start_copy.lock().unwrap();
                            *unlocked_start_copy = Instant::now();
                            let unlocked_dir_copy = dir_copy.lock().unwrap();
                            // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                            for entry in WalkDir::new(unlocked_dir_copy.as_ref().unwrap())
                                    .min_depth(1)
                                    .into_iter()
                                    .filter_map(Result::ok)
                                    .filter(|e| !e.file_type().is_dir()) {
                                // Extract the file extension from the file's name.
                                let file_ext: &OsStr = entry.path().extension().unwrap_or(&default_extension);
                                let show_ext: String = String::from(file_ext.to_string_lossy());
                                let mut unlocked_counts_copy = counts_copy.lock().unwrap();
                                // Add newly encountered file extensions to known file extensions with a counter of 0.
                                let counter: &mut i128 = unlocked_counts_copy.entry(show_ext).or_insert(0);
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
            let mut ext_info: Vec<(&String, &i128)> = unlocked_exts.iter().sorted().collect();
            // Sort file extensions from most to least occurrences, assuming the user wants to see the most numerous filetypes first.
            ext_info.sort_by(|a, b| b.1.cmp(a.1));
            // todo: Optimize table by efficiently displaying viewable rows.
            // Create a scrollable table that (inefficiently) shows all rows, whether they're in the "viewport" or not.
            TableBuilder::new(ui)
                .resizable(true)
                .striped(true)
                .column(Size::initial(150.0).at_least(150.0))
                .column(Size::remainder().at_least(60.0))
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
