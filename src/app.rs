use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui_extras::{Size, TableBuilder};

use crate::catalog_directory;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    extension_counts: HashMap<String, i128>,
    #[serde(skip)]
    total_files: i128,
    picked_path: Option<PathBuf>,
    #[serde(skip)]
    time_taken: Duration,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            extension_counts: HashMap::new(),
            total_files: 0,
            picked_path: None,
            time_taken: Duration::ZERO,
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
            time_taken,
            ..
        } = self;

        // todo: fix live update count (which isn't live)
        // Show a live update of how many files have been summarized.
        *total_files = extension_counts.values().sum();

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
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.picked_path = Some(path);
                    }
                }

                if let Some(picked_path) = &self.picked_path {
                    ui.horizontal(|ui| {
                        ui.label("Chosen directory:");
                        ui.monospace(picked_path.display().to_string());
                    });
                }

                if ui.button("Summarize").clicked() {
                    // Start the stopwatch for summarization time.
                    let now: Instant = Instant::now();
                    catalog_directory(&self.picked_path.as_ref().unwrap(), extension_counts);
                    *time_taken = now.elapsed();
                };

                ui.label(format!(
                    "Summarized {} files in {} milliseconds",
                    &total_files,
                    &time_taken.as_millis()
                ));

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
            // Alphabetize file extensions before occurrence sorting so those with the same count appear alphabetically.
            let mut ext_info: Vec<(&String, &i128)> = extension_counts.iter().sorted().collect();
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
                        body.row(30.0, |mut row| {
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
