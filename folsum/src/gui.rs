//! GUI, which displays inventoried files and their integrity.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dirs::home_dir;
use egui::ViewportCommand;
use egui_extras::{Column, TableBuilder};
#[allow(unused)]
use log::{debug, error, info, trace, warn};
use rfd::FileDialog;

use crate::{DirectoryAuditStatus, FileIntegrity, FoundFile, ManifestCreationStatus, InventoryStatus, audit_directory_inventory};
use crate::{export_csv, inventory_directory};

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // Define default fields when deserializing old state.
pub struct FolsumGui {
    // Unique file extensions and the number of times each one was encountered.
    #[serde(skip)]
    inventoried_files: Arc<Mutex<Vec<FoundFile>>>,
    // Number of files inventoried, which doesn't include files and directories that were skipped.
    #[serde(skip)]
    total_files: u32,
    // User's chosen directory that will be recursively inventories when the "inventory" button's clicked.
    chosen_inventory_path: Arc<Mutex<Option<PathBuf>>>,
    // User's chosen manifest file that we generated previously.
    chosen_manifest: Arc<Mutex<Option<PathBuf>>>,
    // Time that directory inventory starts so it can be used to calculate the time taken.
    #[serde(skip)]
    inventory_start: Arc<Mutex<Instant>>,
    // Amount of time that it's taken to inventory a directory.
    #[serde(skip)]
    time_taken: Arc<Mutex<Duration>>,
    #[serde(skip)]
    inventory_status: Arc<Mutex<InventoryStatus>>,
    #[serde(skip)]
    directory_audit_status: Arc<Mutex<DirectoryAuditStatus>>,
    #[serde(skip)]
    manifest_creation_status: Arc<Mutex<ManifestCreationStatus>>,
}

impl Default for FolsumGui {
    fn default() -> Self {
        Self {
            inventoried_files: Arc::new(Mutex::new(vec![])),
            total_files: 0,
            chosen_inventory_path: Arc::new(Mutex::new(None)),
            chosen_manifest: Arc::new(Mutex::new(None)),
            inventory_start: Arc::new(Mutex::new(Instant::now())),
            time_taken: Arc::new(Mutex::new(Duration::ZERO)),
            inventory_status: Arc::new(Mutex::new(InventoryStatus::NotStarted)),
            directory_audit_status: Arc::new(Mutex::new(DirectoryAuditStatus::Unaudited)),
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
            inventoried_files,
            total_files,
            chosen_inventory_path,
            chosen_manifest,
            inventory_start,
            time_taken,
            inventory_status,
            directory_audit_status,
            manifest_creation_status,
            ..
        } = self;

        // Update the count of total files inventoried.
        *total_files = inventoried_files.lock().unwrap().len() as u32;
        // Update the screen on each iteration, bounded by the refresh rate of the user's screen.
        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Add a menu bar to the top of the screen.
            egui::menu::bar(ui, |ui| {
                // Include a File->Quit menu item.
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

                // todo: Add a reset button for text zoom.

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

                // Define the "First..." section in the left pane.
                ui.horizontal(|ui| {
                    ui.label("First,");

                    if ui.button("choose").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            info!("User chose inventory directory: {:?}", path);
                            *chosen_inventory_path = Arc::new(Mutex::new(Some(path)));
                        }
                    }

                    ui.label("a folder to ");

                    // Check whether the user has selected a directory to inventory.
                    let locked_chosen_inventory_path = chosen_inventory_path.lock().unwrap();
                    let chosen_inventory_path_copy = locked_chosen_inventory_path.clone();
                    drop(locked_chosen_inventory_path);

                    // Grey out the "audit" button until the user has selected a directory to inventory.
                    if ui.add_enabled(chosen_inventory_path_copy.is_some(), egui::Button::new("inventory")).clicked() {
                        info!("User started discovery manifest creation");
                        let _result = inventory_directory(
                            &chosen_inventory_path,
                            &inventoried_files,
                            &inventory_start,
                            &time_taken,
                            &inventory_status,
                            &directory_audit_status,
                            &manifest_creation_status,
                        );
                    };

                    ui.label("and create a manifest from.");
                });

                ui.label("A manifest file describing the folder's contents will be exported to the folder that was inventoried.");

                ui.horizontal(|ui| {
                    // Check if the user has picked a directory to inventory.
                    let locked_path: &Option<PathBuf> = &*chosen_inventory_path.lock().unwrap();
                    let shown_path: &str = match &*locked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No folder selected",
                    };
                    ui.label("Chosen folder:");
                    // Display the user's chosen directory in monospace font.
                    ui.monospace(shown_path);
                });


                // Show the inventory status to the user.
                ui.horizontal(|ui| {
                    let locked_inventory_status = inventory_status.lock().unwrap();
                    let inventory_status_copy = locked_inventory_status.clone();
                    drop(locked_inventory_status);

                    let display_inventory_status = match inventory_status_copy {
                        InventoryStatus::NotStarted => "not started.",
                        InventoryStatus::InProgress => "in progress.",
                        InventoryStatus::Done => "completed.",
                    };

                    ui.label(format!("Inventory {display_inventory_status}"));
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
                        "Inventoried {} files in {} milliseconds",
                        &total_files,
                        &locked_time_taken.as_millis()
                    ));
                });

                // Check whether the user has selected a directory to inventory.
                let locked_chosen_inventory_path = chosen_inventory_path.lock().unwrap();
                let chosen_inventory_path_copy = locked_chosen_inventory_path.clone();
                drop(locked_chosen_inventory_path);

                let export_prerequisites_met = export_prerequisites_met(&chosen_inventory_path_copy, &inventory_status, &manifest_creation_status);

                // If we're ready to export a manifest file, then do so.
                if export_prerequisites_met {
                    let _result = export_csv(&inventoried_files, &manifest_creation_status, &chosen_inventory_path);
                };

                ui.separator();

                ui.heading("Perform Audit");

                // Directory audit section.
                ui.vertical(|ui| {
                    // Directory audit text block.
                    ui.horizontal(|ui| {
                        ui.label("Second,");

                        // Grey out/disable the "select" file picker button if manifest selection prerequisites aren't met.
                        if ui.add_enabled(inventory_is_complete(inventory_status.clone()),
                                          // Prompt the user to choose a FolSum manifest to verify against.
                                          egui::Button::new("select")).clicked() {
                            // Open the manifest file chooser in the same directory that was inventoried.
                            let starting_directory = chosen_inventory_path.lock().unwrap().clone().unwrap_or_else(|| {
                                // Assume that an inventory directory has been selected b/c prereqs were met.
                                let error_message = "Expected an inventory directory to be selected";
                                error!("{}", error_message);
                                // Default to the user's home dir for now b/c we don't have good error propagation yet.
                                home_dir().unwrap()
                            });
                            // Open the file picker for the manifest file.
                            if let Some(path) = FileDialog::new()
                                // Show only `.csv` files b/c a shortcoming of rfd is that we can't filter for `.folsum.csv`.
                                .add_filter("CSV", &["csv"])
                                .set_title("Choose FolSum CSV manifest file as an audit rubric")
                                .set_directory(starting_directory)
                                .pick_file() {
                                info!("User chose manifest file: {:?}", path);
                                *chosen_manifest = Arc::new(Mutex::new(Some(path)));
                            }

                            info!("🏁 User started audit");
                            audit_directory_inventory(&inventoried_files,
                                                      &directory_audit_status,
                                                      &manifest_creation_status).unwrap();

                        }
                        ui.label("a previously-generated manifest to verify against.");
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("Chosen manifest:");

                    let locked_chosen_manifest = chosen_manifest.lock().unwrap();
                    let chosen_manifest_copy = locked_chosen_manifest.clone();
                    drop(locked_chosen_manifest);

                    let shown_path = match chosen_manifest_copy {
                        Some(ref found_previous_manifest) => {
                            let manifest_filename = found_previous_manifest.file_name().unwrap();
                            manifest_filename.to_string_lossy().to_string()
                        },
                        None => "No manifest file has been chosen".to_string(),
                    };

                    // Display the previous manifest file's path in monospace font.
                    ui.monospace(shown_path);
                });


                // Show the user where we are in the directory audit process.
                ui.horizontal(|ui| {
                    let locked_directory_audit_status = directory_audit_status.lock().unwrap();
                    let directory_audit_status_copy = locked_directory_audit_status.clone();
                    drop(locked_directory_audit_status);
                    let shown_directory_audit_status = match directory_audit_status_copy {
                        DirectoryAuditStatus::Unaudited => "not started.",
                        DirectoryAuditStatus::InProgress => "in progress...",
                        DirectoryAuditStatus::Audited => "complete. Data integrity verified.",
                        DirectoryAuditStatus::DiscrepanciesFound => "complete. Data integrity compromised.",
                    };

                    // Display folder verification progress.
                    ui.label(format!("Folder audit {shown_directory_audit_status}"));
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
                ui.heading("Folder Inventory");
                ui.separator();
            });

            // todo: Sort paths alphabetically before displaying in table.
            let file_paths_locked = inventoried_files.lock().unwrap();

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
                        ui.heading("Audit Finding");
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
                                let display_file_integrity = match &found_file.file_integrity {
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
                                ui.label(display_file_integrity);
                            });
                        });
                    }
                });
        });
    }
}

/// Check if inventory is done.
fn inventory_is_complete(inventory_status: Arc<Mutex<InventoryStatus>>) -> bool {
    let locked_inventory_status = inventory_status.lock().expect("Lock poisoned");
    let inventory_status_copy = locked_inventory_status.clone();
    drop(locked_inventory_status);

    let inventory_complete = match inventory_status_copy {
        InventoryStatus::NotStarted => {
            trace!("❌ Nothing has been inventoried, so nothing can be audited");
            false
        }
        InventoryStatus::InProgress => {
            trace!("❌ In progress inventory means that nothing can be audited");
            false
        }
        InventoryStatus::Done => {
            trace!("✅ Data in inventory table, so audit can proceed");
            true
        }
    };
    inventory_complete
}

// Decide whether we're ready to create an export file.
fn export_prerequisites_met(chosen_inventory_path_copy: &Option<PathBuf>,
                            inventory_status: &Arc<Mutex<InventoryStatus>>,
                            manifest_creation_status: &Arc<Mutex<ManifestCreationStatus>>) -> bool {
    let inventory_is_complete = inventory_is_complete(inventory_status.clone());

    let inventory_path_selected = chosen_inventory_path_copy.is_some();

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

    let export_prerequisites_met = inventory_is_complete
        && inventory_path_selected
        && manifest_creator_ready;

    if export_prerequisites_met {
        trace!("Decided that the prerequisites for an export were met.");
    } else {
        trace!("Decided that the prerequisites for an export were not met.");
    };
    export_prerequisites_met
}
