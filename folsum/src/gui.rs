use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui_extras::{Column, TableBuilder};

use dirs::home_dir;
use egui::ViewportCommand;
use rfd::FileDialog;
#[allow(unused)]
use log::{debug, error, info, trace, warn};

use crate::{DirectoryVerificationStatus, FileIntegrity, FoundFile, ManifestCreationStatus, SummarizationStatus, audit_summarization};
use crate::{export_csv, summarize_directory};

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Define default fields when deserializing old state.
pub struct FolsumGui {
    // Unique file extensions and the number of times each one was encountered.
    #[serde(skip)]
    file_paths: Arc<Mutex<Vec<FoundFile>>>,
    // Number of files summarized, which doesn't include files and directories that were skipped.
    #[serde(skip)]
    total_files: u32,
    // User's chosen directory that will be recursively summarized when the "Summarize" button's clicked.
    summarization_path: Arc<Mutex<Option<PathBuf>>>,
    // User's chosen manifest file that we generated previously.
    #[serde(skip)]
    chosen_manifest: Arc<Mutex<Option<PathBuf>>>,
    // Time that summarization starts so it can be used to calculate the time taken.
    #[serde(skip)]
    summarization_start: Arc<Mutex<Instant>>,
    // Amount of time that it takes to summarize a directory.
    #[serde(skip)]
    time_taken: Arc<Mutex<Duration>>,
    #[serde(skip)]
    summarization_status: Arc<Mutex<SummarizationStatus>>,
    #[serde(skip)]
    directory_verification_status: Arc<Mutex<DirectoryVerificationStatus>>,
    #[serde(skip)]
    manifest_creation_status: Arc<Mutex<ManifestCreationStatus>>,
}

impl Default for FolsumGui {
    fn default() -> Self {
        Self {
            file_paths: Arc::new(Mutex::new(vec![])),
            total_files: 0,
            summarization_path: Arc::new(Mutex::new(None)),
            chosen_manifest: Arc::new(Mutex::new(None)),
            summarization_start: Arc::new(Mutex::new(Instant::now())),
            time_taken: Arc::new(Mutex::new(Duration::ZERO)),
            summarization_status: Arc::new(Mutex::new(SummarizationStatus::NotStarted)),
            directory_verification_status: Arc::new(Mutex::new(DirectoryVerificationStatus::Unverified)),
            manifest_creation_status: Arc::new(Mutex::new(ManifestCreationStatus::NotStarted)),
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
            file_paths,
            total_files,
            summarization_path,
            chosen_manifest,
            summarization_start,
            time_taken,
            summarization_status,
            directory_verification_status,
            manifest_creation_status,
            ..
        } = self;

        // Update the count of total files summarized.
        *total_files = file_paths.lock().unwrap().len() as u32;
        // Update the screen on each iteration, bounded by the refresh rate of the user's screen.
        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Add a menu bar to the top of the screen.
            egui::menu::bar(ui, |ui| {
                // Don't include a File->Quit menu item when compiling for web.
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
                // Add a dark/light mode toggle button to the top menu bar.
                egui::widgets::global_theme_preference_switch(ui);

                // Add a menu bar button that decreases zoom.
                if ui.add(egui::Button::new("-")).on_hover_text("Decrease zoom").clicked() {
                    let current_zoom_factor = ctx.zoom_factor();
                    let new_zoom_factor = current_zoom_factor - 0.1;
                    ctx.set_zoom_factor(new_zoom_factor);

                    info!("User decreased zoom from {current_zoom_factor:?}\
                           to {new_zoom_factor:?}");
                };

                // todo: Add a text reset button.

                // Add a menu bar button that increases zoom.
                if ui.add(egui::Button::new("+")).on_hover_text("Increase zoom").clicked() {
                    let current_zoom_factor = ctx.zoom_factor();
                    let new_zoom_factor = current_zoom_factor + 0.1;
                    ctx.set_zoom_factor(new_zoom_factor);

                    info!("User increased zoom from {current_zoom_factor:?}\
                           to {new_zoom_factor:?}");
                };
            });
        });

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Make Discovery");

                // Define the "First.." section in the left pane.
                ui.horizontal(|ui| {
                    ui.label("First,");

                    if ui.button("choose").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            info!("User chose summarization directory: {:?}", path);
                            *summarization_path = Arc::new(Mutex::new(Some(path)));
                        }
                    }

                    ui.label("a folder to ");

                    // Check whether the user has selected a directory to summarize.
                    let locked_summarization_path = summarization_path.lock().unwrap();
                    let summarization_path_actual = locked_summarization_path.clone();
                    drop(locked_summarization_path);

                    // Grey out the "audit" button until the user has selected a directory to summarize.
                    if ui.add_enabled(summarization_path_actual.is_some(), egui::Button::new("audit")).clicked() {
                        info!("User started discovery manifest creation");
                        let _result = summarize_directory(
                            &summarization_path,
                            &file_paths,
                            &summarization_start,
                            &time_taken,
                            &summarization_status,
                            &directory_verification_status,
                            &manifest_creation_status,
                        );
                    };

                    ui.label("and create a manifest from.");
                });

                ui.label("A manifest file containing audit results will be exported to the folder that was audited.");

                ui.horizontal(|ui| {
                    // Check if the user has picked a directory to summarize.
                    let locked_path: &Option<PathBuf> = &*summarization_path.lock().unwrap();
                    let shown_path: &str = match &*locked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No folder selected",
                    };
                    ui.label("Chosen folder:");
                    // Display the user's chosen directory in monospace font.
                    ui.monospace(shown_path);
                });


                // Show the summarization status to the user.
                ui.horizontal(|ui| {
                    let locked_summarization_status = summarization_status.lock().unwrap();
                    let summarization_status_copy = locked_summarization_status.clone();
                    drop(locked_summarization_status);

                    let display_summarization_status = match summarization_status_copy {
                        SummarizationStatus::NotStarted => "not started.",
                        SummarizationStatus::InProgress => "in progress.",
                        SummarizationStatus::Done => "completed.",
                    };

                    ui.label(format!("Audit {display_summarization_status}"));
                });

                // Show the manifest file creation/export status to the user.
                ui.horizontal(|ui| {
                    let locked_manifest_creation_status = manifest_creation_status.lock().unwrap();
                    let manifest_creation_status_copy = locked_manifest_creation_status.clone();
                    drop(locked_manifest_creation_status);

                    let display_manifest_creation_status = match manifest_creation_status_copy {
                        ManifestCreationStatus::NotStarted => "not started.".to_string(),
                        ManifestCreationStatus::InProgress => "in progress.".to_string(),
                        ManifestCreationStatus::Done(manifest_file_path) => {
                            let manifest_filename = manifest_file_path.file_name().unwrap();
                            let display_manifest_filename = manifest_filename.to_string_lossy().to_string();
                            format!("completed: {display_manifest_filename}")
                        },
                    };

                    ui.label(format!("Manifest file creation {display_manifest_creation_status}"));
                });

                ui.horizontal(|ui| {
                    let locked_time_taken = time_taken.lock().unwrap();
                    ui.label(format!(
                        "Audited {} files in {} milliseconds",
                        &total_files,
                        &locked_time_taken.as_millis()
                    ));
                });

                // Check whether the user has selected a directory to summarize.
                let locked_summarization_path = summarization_path.lock().unwrap();
                let summarization_path_copy = locked_summarization_path.clone();
                drop(locked_summarization_path);

                let export_prerequisites_met = export_prerequisites_met(&summarization_path_copy, &summarization_status, &manifest_creation_status);

                // If we're ready to export a verification manifest file, then do so.
                if export_prerequisites_met {
                    let _result = export_csv(&file_paths, &manifest_creation_status, &summarization_path);
                };

                ui.separator();

                ui.heading("Verify a Folder");

                // Folder verification section.
                ui.vertical(|ui| {
                    // If everything's ready to verify...
                    let verification_prerequisites_met = summarization_is_complete(summarization_status.clone());

                    // Verification text block.
                    ui.horizontal(|ui| {
                        ui.label("Second,");

                        // Grey out/disable the "select" button if manifest selection prerequisites aren't met.
                        if ui.add_enabled(verification_prerequisites_met,
                                          // Prompt the user to choose a FolSum manifest to verify against.
                                          egui::Button::new("select")).clicked() {
                            // Open the "select manifest file" dialog.
                            let starting_directory = match summarization_path.lock().unwrap().clone() {
                                // Open the verification file chooser in the same dir as the previous export.
                                Some(verification_file) => verification_file.parent().unwrap().to_path_buf(),
                                // Otherwise, if there was no previous verification file, then open the export dialog in the user's home dir.
                                None => home_dir().expect("Failed to get user's home directory"),
                            };
                            if let Some(path) = FileDialog::new()
                                // Show only `.csv` files b/c a shortcoming of rfd is that we can't filter for `.folsum.csv`.
                                .add_filter("CSV", &["csv"])
                                .set_title("Choose FolSum CSV file to verify against")
                                // Open manifest file picker in the same directory that was selected for summarization.
                                .set_directory(starting_directory)
                                .pick_file() {
                                info!("User chose verification file: {:?}", path);
                                *chosen_manifest = Arc::new(Mutex::new(Some(path)));
                            }

                            info!("🏁 User started verification");
                            audit_summarization(&file_paths,
                                                &directory_verification_status,
                                                &manifest_creation_status).unwrap();

                        }
                        ui.label("a previously-generated manifest to verify against.");
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("Chosen manifest:");

                    let locked_previous_manifest = chosen_manifest.lock().unwrap();
                    let previous_manifest_copy = locked_previous_manifest.clone();
                    drop(locked_previous_manifest);

                    let shown_path = match previous_manifest_copy {
                        Some(ref found_previous_manifest) => {
                            let manifest_filename = found_previous_manifest.file_name().unwrap();
                            manifest_filename.to_string_lossy().to_string()
                        },
                        None => "No manifest file has been chosen".to_string(),
                    };

                    // Display the previous manifest file's path in monospace font.
                    ui.monospace(shown_path);
                });


                ui.horizontal(|ui| {
                    let locked_directory_verification_status = directory_verification_status.lock().unwrap();
                    let directory_verification_status_copy = locked_directory_verification_status.clone();
                    drop(locked_directory_verification_status);
                    let shown_directory_verification_status = match directory_verification_status_copy {
                        DirectoryVerificationStatus::Unverified => "not started.",
                        DirectoryVerificationStatus::InProgress => "in progress...",
                        DirectoryVerificationStatus::Verified => "complete. Data integrity verified.",
                        DirectoryVerificationStatus::VerificationFailed => "complete. Data integrity compromised.",
                    };

                    // Display folder verification progress.
                    ui.label(format!("Folder verification {shown_directory_verification_status}"));
                });

                ui.separator();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("written with love by Brooke Deuson ");
                        ui.label("(");
                        ui.hyperlink_to("goingforbrooke", "https://goingforbrooke.com");
                        ui.label(") ");
                        ui.label("for ");
                        ui.hyperlink_to("Trafficking Free Tomorrow", "https://traffickingfreetomorrow.com");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Summarization by File Extension");
                ui.separator();
            });

            // todo: Sort paths alphabetically before displaying in table.
            let file_paths_locked = file_paths.lock().unwrap();

            // todo: Optimize table display by efficiently displaying viewable rows with `show_rows()`.
            // Create a scrollable table that (inefficiently) shows all rows, whether they're in the "viewport" or not.
            TableBuilder::new(ui)
                .resizable(true)
                .striped(true)
                .column(Column::initial(150.0).at_least(150.0))
                .column(Column::initial(200.0).at_least(60.0))
                .column(Column::remainder().at_least(60.0))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("File Path");
                    });
                    header.col(|ui| {
                        ui.heading("MD5 Hash");
                    });
                    header.col(|ui| {
                        ui.heading("Verification Status");
                    });
                })
                .body(|mut body| {
                    for found_file in file_paths_locked.iter() {
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                let show_path: String = String::from(found_file.file_path.to_string_lossy());
                                ui.label(show_path);
                            });
                            row.col(|ui| {
                                ui.label(found_file.md5_hash.clone());
                            });
                            row.col(|ui| {
                                let display_verification_status = match &found_file.file_verification_status {
                                    FileIntegrity::Unverified => "Unverified",
                                    FileIntegrity::InProgress => "Verifying...",
                                    FileIntegrity::Verified(_) => "Verified",
                                    FileIntegrity::VerificationFailed(integrity_detail) => {
                                        // If the file's missing...
                                        if !integrity_detail.file_path_matches {
                                            "Failed verification: file missing"
                                        // Otherwise, if the file's MD5 hash doesn't match...
                                        } else if !integrity_detail.md5_hash_matches {
                                            "Failed verification: MD5 hash mismatch"
                                        } else {
                                            "Failed verification: unknown reason"
                                        }
                                    }
                                };
                                ui.label(display_verification_status);
                            });
                        });
                    }
                });
        });
    }
}

/// Check if summarization is done.
fn summarization_is_complete(summarization_status: Arc<Mutex<SummarizationStatus>>) -> bool {
    let locked_summarization_status = summarization_status.lock().expect("Lock poisoned");
    let summarization_status_copy = locked_summarization_status.clone();
    drop(locked_summarization_status);
    let summarization_complete = match summarization_status_copy {
        SummarizationStatus::NotStarted => {
            trace!("❌ Nothing has been summarized, so nothing can be verified");
            false
        }
        SummarizationStatus::InProgress => {
            trace!("❌ In progress summarization means that nothing can be verified");
            false
        }
        SummarizationStatus::Done => {
            trace!("✅ Data in summarization table, so verification can proceed");
            true
        }
    };
    summarization_complete
}

// Decide whether we're ready to create an export file.
fn export_prerequisites_met(summarization_path_copy: &Option<PathBuf>,
                            summarization_status: &Arc<Mutex<SummarizationStatus>>,
                            manifest_creation_status: &Arc<Mutex<ManifestCreationStatus>>) -> bool {
    let summarization_complete = summarization_is_complete(summarization_status.clone());

    let summarization_path_selected = summarization_path_copy.is_some();

    let locked_manifest_creation_status = manifest_creation_status.lock().unwrap();
    let manifest_creation_status_copy = locked_manifest_creation_status.clone();
    drop(locked_manifest_creation_status);
    let manifest_creator_ready = match manifest_creation_status_copy {
        ManifestCreationStatus::NotStarted => true,
        // Don't interrupt or overwrite an export that's in-progress.
        ManifestCreationStatus::InProgress => false,
        // Don't repeatedly create a new manifest export.
        ManifestCreationStatus::Done(_) => false,
    };

    let export_prerequisites_met = summarization_complete
        && summarization_path_selected
        && manifest_creator_ready;

    if export_prerequisites_met {
        trace!("Decided that the prerequisites for an export were met.");
    } else {
        trace!("Decided that the prerequisites for an export were not met.");
    };
    export_prerequisites_met
}
