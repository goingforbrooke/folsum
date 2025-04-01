// Std crates for macOS, Windows, *and* WASM builds.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Std crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::time::SystemTime;
#[cfg(any(target_family = "unix", target_family = "windows"))]
use std::time::{Duration, Instant};

// External crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use chrono::{DateTime, Local};
#[cfg(any(target_family = "unix", target_family = "windows"))]
use egui::ViewportCommand;
use egui_extras::{Column, TableBuilder};
#[cfg(any(target_family = "unix", target_family = "windows"))]
use rfd::FileDialog;

// External crates for macOS, Windows, *and* WASM builds.
#[allow(unused)]
use log::{debug, error, info, trace, warn};

// External crates for WASM builds.
#[cfg(target_family = "wasm")]
use web_time::{Duration, Instant};

// Internal crates for macOS, Windows, *and* WASM builds.
use crate::{DirectoryVerificationStatus, FILEDATE_PREFIX_FORMAT, FileIntegrity, FoundFile, verify_summarization};

// Internal crates for macOS and Windows builds.
#[cfg(any(target_family = "unix", target_family = "windows"))]
use crate::{export_csv, summarize_directory, ManifestCreationStatus, SummarizationStatus};

// Internal crates for WASM builds.
#[cfg(target_family = "wasm")]
use crate::wasm_demo_summarize_directory;

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
    verification_file_path: Arc<Mutex<Option<PathBuf>>>,
    // User's chosen directory and filename for CSV exports.
    export_file: Arc<Mutex<Option<PathBuf>>>,
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
            verification_file_path: Arc::new(Mutex::new(None)),
            export_file: Arc::new(Mutex::new(None)),
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
            #[cfg(any(target_family = "unix", target_family = "windows"))]
            summarization_path,
            #[cfg(any(target_family = "unix", target_family = "windows"))]
            verification_file_path,
            #[cfg(any(target_family = "unix", target_family = "windows"))]
            export_file,
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
                #[cfg(any(target_family = "unix", target_family = "windows"))]
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

                    // Don't add a directory picker when compiling for web.
                    #[cfg(any(target_family = "unix", target_family = "windows"))]
                    if ui.button("choose").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            info!("User chose summarization directory: {:?}", path);
                            *summarization_path = Arc::new(Mutex::new(Some(path)));
                        }
                    }

                    ui.label("a folder to ");

                    if ui.button("create").clicked() {
                        info!("User started discovery manifest creation");
                        #[cfg(any(target_family = "unix", target_family = "windows"))]
                            let _result = summarize_directory(
                            &summarization_path,
                            &file_paths,
                            &summarization_start,
                            &time_taken,
                            &summarization_status,
                            &directory_verification_status,
                        );
                        #[cfg(target_family = "wasm")]
                            let _result = wasm_demo_summarize_directory(
                            &file_paths,
                            &summarization_start,
                            &time_taken,
                            &summarization_status,
                        );
                    };

                    ui.label("a discovery manifest from.");
                });

                ui.label("A discovery manifest file will be exported to the folder that was assessed.");

                ui.horizontal(|ui| {
                    // Check if the user has picked a directory to summarize.
                    #[cfg(any(target_family = "unix", target_family = "windows"))]
                        let locked_path: &Option<PathBuf> = &*summarization_path.lock().unwrap();
                    #[cfg(any(target_family = "unix", target_family = "windows"))]
                        let shown_path: &str = match &*locked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No folder selected",
                    };
                    #[cfg(target_family = "wasm")]
                        let shown_path = "N/A";
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

                    ui.label(format!("Discovery assessment {display_summarization_status}"));
                });

                // Show the manifest file creation/export status to the user.
                ui.horizontal(|ui| {
                    let locked_manifest_creation_status = manifest_creation_status.lock().unwrap();
                    let manifest_creation_status_copy = locked_manifest_creation_status.clone();
                    drop(locked_manifest_creation_status);

                    let display_manifest_creation_status = match manifest_creation_status_copy {
                        ManifestCreationStatus::NotStarted => "not started.",
                        ManifestCreationStatus::InProgress => "in progress.",
                        ManifestCreationStatus::Done => "completed.",
                    };

                    ui.label(format!("Manifest file creation {display_manifest_creation_status}"));
                });

                ui.horizontal(|ui| {
                    let locked_time_taken = time_taken.lock().unwrap();
                    ui.label(format!(
                        "Examined {} files in {} milliseconds",
                        &total_files,
                        &locked_time_taken.as_millis()
                    ));
                });

                #[cfg(any(target_family = "unix", target_family = "windows"))]
                ui.horizontal(|ui| {
                    // Check whether the user has selected a directory to summarize.
                    let summarization_path = summarization_path.lock().unwrap();
                    let summarization_path_copy = summarization_path.clone();
                    drop(summarization_path);

                    let export_prerequisites_met = export_prerequisites_met(&summarization_path_copy, &summarization_status);

                    // If we're ready to export a verification manifest, then do so.
                    if export_prerequisites_met {
                        let date_today: DateTime<Local> = DateTime::from(SystemTime::now());
                        // Prefix the export filename with the non-zero padded date and time.
                        let formatted_date = date_today.format(FILEDATE_PREFIX_FORMAT).to_string();

                        // Extract the name of the summarized directory so we can use it to name the export file.
                        // Assume that a directory's been selected b/c we checked in the export prerequisites before this.
                        let summarization_path_copy = summarization_path_copy.unwrap();
                        let raw_directory_name = summarization_path_copy.file_name().unwrap();
                        let display_directory_name = raw_directory_name.to_string_lossy().to_string();

                        // Name the export file `YY-MM-DD-HH-MM_<summarized folder name>.folsum.csv`. (we'll add the .csv later).
                        let export_filename = format!("{formatted_date}_{display_directory_name}.folsum");
                        // Put the export file into the directory that was assessed.
                        let export_path: PathBuf = [summarization_path_copy, PathBuf::from(export_filename)].iter().collect();
                        debug!("Created path for new export file: {export_path:?}");

                        *export_file = Arc::new(Mutex::new(Some(export_path)));
                        let _result = export_csv(&export_file, &file_paths);
                    };
                });

                ui.separator();

                #[cfg(target_family = "wasm")]
                {
                    ui.vertical(|ui| {
                        ui.heading("Behold the Power of WASM! 🦀");
                        ui.label("".to_string());
                        let wasm_message = "This is a Rust binary running inside of your browser's sandbox! The MacOS or Windows version look the same, but with a button that lets you choose a folder to summarize.";
                        ui.label(wasm_message.to_string());

                        ui.separator();

                        ui.heading("Differences with Executables ⚖️");
                        ui.label("".to_string());
                        let usage_message = "We can't summarize the contents of files on your computer. Why? Because that's how your browser protects you from the internet. 👻";
                        ui.label(usage_message.to_string());

                        ui.separator();

                        ui.heading("Code Source 👩🏼‍💻");
                        ui.label("".to_string());
                        let repo_message = "The Rust code powering this can be found here: ";
                        ui.label(repo_message.to_string());

                        ui.hyperlink_to("github.com/goingforbrooke/folsum", "https://github.com/goingforbrooke/folsum");

                        ui.separator();
                    });
                }

                #[cfg(any(target_family = "unix", target_family = "windows"))]
                ui.heading("Verify a Folder");


                // Folder verification section.
                ui.vertical(|ui| {
                    // Check if summarization table has data.
                    let file_paths_locked = file_paths.lock().unwrap();
                    let file_paths_copy = file_paths_locked.clone();
                    drop(file_paths_locked);
                    let summarization_table_has_data = match file_paths_copy.is_empty() {
                        false => {
                            trace!("✅ GUI table has data");
                            true
                        },
                        true => {
                            trace!("❌ GUI table has no data");
                            false
                        },
                    };
                    // If everything's ready to verify...
                    // todo: Add verification prerequisite: export file must be selected.
                    let verification_prerequisites_met = summarization_table_has_data && summarization_is_complete(summarization_status.clone());

                    // Verification text block.
                    ui.horizontal(|ui| {
                        ui.label("Second,");
                        // Grey out/disable the "verify" button if summarization prerequisites aren't met.
                        if ui.add_enabled(verification_prerequisites_met, egui::Button::new("verify")).clicked() {
                            info!("🏁 User started verification");
                            // ... then ensure that its contents match the verification file.
                            verify_summarization(&file_paths, &summarization_path, &directory_verification_status).unwrap();
                        }
                        ui.label("the folder's contents against the most recent FolSum manifest.");
                    });
                    ui.label("FolSum looks for manifests inside of the folder that was summarized.");
                });

                ui.horizontal(|ui| {
                    // Check if the user has picked a FolSum CSV to verify against.
                    #[cfg(any(target_family = "unix", target_family = "windows"))]
                        let locked_path: &Option<PathBuf> = &*verification_file_path.lock().unwrap();
                    #[cfg(any(target_family = "unix", target_family = "windows"))]
                        let shown_path: &str = match &*locked_path {
                        Some(the_path) => the_path.as_os_str().to_str().unwrap(),
                        None => "No verification file selected",
                    };
                    #[cfg(target_family = "wasm")]
                        let shown_path = "N/A";
                    ui.label("Verification file:");
                    // Display the user's chosen directory in monospace font.
                    ui.monospace(shown_path);
                });


                #[cfg(any(target_family = "unix", target_family = "windows"))]
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

                #[cfg(any(target_family = "unix", target_family = "windows"))]
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

/// Decide whether we're ready to export.
fn export_prerequisites_met(summarization_path_copy: &Option<PathBuf>, summarization_status: &Arc<Mutex<SummarizationStatus>>) -> bool {
    // Decide whether we're ready to create an export file.
    let summarization_complete = summarization_is_complete(summarization_status.clone());
    let summarization_path_selected = summarization_path_copy.is_some();
    let export_prerequisites_met = summarization_complete && summarization_path_selected;

    if export_prerequisites_met {
        trace!("Decided that the prerequisites for an export were met.");
    } else {
        trace!("Decided that the prerequisites for an export were not met.");
    };
    export_prerequisites_met
}
