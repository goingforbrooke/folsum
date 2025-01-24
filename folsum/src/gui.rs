use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Local};
#[cfg(not(target_arch = "wasm32"))]
use dirs::home_dir;
#[cfg(not(target_arch = "wasm32"))]
use egui::ViewportCommand;
use egui_extras::{Column, TableBuilder};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

#[cfg(target_arch = "wasm32")]
use web_time::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

#[allow(unused)]
use log::{debug, error, info, trace, warn};
#[cfg(not(target_arch = "wasm32"))]
use crate::{export_csv, summarize_directory};
use crate::sort_counts;


// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Define default fields when deserializing old state.
pub struct FolsumGui {
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

impl Default for FolsumGui {
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

impl FolsumGui {
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

impl eframe::App for FolsumGui {
    // Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            extension_counts,
            total_files,
            summarization_path,
            #[cfg(not(target_arch = "wasm32"))]
            export_file,
            #[cfg(not(target_arch = "wasm32"))]
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
                        ctx.send_viewport_cmd(ViewportCommand::Close);
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
                        info!("User chose summarization directory: {:?}", path);
                        *summarization_path = Arc::new(Mutex::new(Some(path)));
                    }
                }

                ui.horizontal(|ui| {
                    let locked_path: &Option<PathBuf> = &*summarization_path.lock().unwrap();
                    // Check if the user has picked a directory to summarize.
                    let shown_path: &str = match &*locked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No directory selected",
                    };
                    ui.label("Chosen directory:");
                    // Display the user's chosen directory in monospace font.
                    ui.monospace(shown_path);
                });

                ui.separator();

                if ui.button("Summarize").clicked() {
                    info!("User started summarization");
                    #[cfg(not(target_arch = "wasm32"))]
                    let _result = summarize_directory(
                        &summarization_path,
                        &extension_counts,
                        &summarization_start,
                        &time_taken,
                    );
                };

                ui.horizontal(|ui| {
                    let locked_time_taken = time_taken.lock().unwrap();
                    ui.label(format!(
                        "Summarized {} files in {} milliseconds",
                        &total_files,
                        &locked_time_taken.as_millis()
                    ));
                });

                ui.separator();

                #[cfg(target_arch = "wasm32")]
                {
                    ui.vertical(|ui| {
                        let wasm_message = "Behold the power of WASM! 🦀 This is a Rust binary running inside of your browser's sandbox! It looks and functions exactly the same way on MacOS and Windows.";
                        ui.label(wasm_message.to_string());

                        ui.separator();

                        let usage_message = "Since we're in a browser, we can't summarize the contents of files on your computer.";
                        ui.label(usage_message.to_string());
                    });
                }

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
                        None => home_dir().expect("Failed to get user's home directory"),
                    };
                    trace!("Found user's home directory: {:?}", &starting_directory);
                    // Ask user where they'd like to save the CSV export and what they'd like it to be called.
                    if let Some(path) = FileDialog::new()
                        // Add `.csv` to the end of the user's chosen name for the CSV export.
                        .add_filter("csv", &["csv"])
                        .set_title("Export extension counts to CSV file")
                        // Open export dialogs in the last saved directory (if it exists), otherwise in the user's home directory.
                        .set_directory(starting_directory)
                        // Set the default filename for CSV exports to YY_MM_DD_folsum_export.
                        .set_file_name(&export_filename)
                        .save_file()
                    {
                        *export_file = Arc::new(Mutex::new(Some(path)));
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let _result = export_csv(&export_file, &extension_counts);
                };

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("written with love by ");
                        ui.hyperlink_to("goingforbrooke", "https://goingforbrooke.com");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Summarization by File Extension");
                ui.separator();
            });
            let locked_exts = extension_counts.lock().unwrap();
            // Sort extension counts in descending order, then alphabetically.
            let ext_info = sort_counts(&*locked_exts);
            // todo: Optimize table display by efficiently displaying viewable rows with `show_rows()`.
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
