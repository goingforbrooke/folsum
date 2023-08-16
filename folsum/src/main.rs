// Standard library libraries.
use std::collections::HashMap;
use std::env::current_dir;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::sync::{Arc, RwLock};

// Iced GUI libraries.
use iced::executor;
use iced::widget::{button, container, text, Column, Row, scrollable};
use iced::{Application, Command, Element, Length, Settings, Theme, Subscription};
use iced::futures::channel::mpsc;

use iced::subscription::channel;
use iced::futures::sink::SinkExt;
use iced::futures::stream::StreamExt;

use itertools::Itertools;

// Third-party libraries.
use walkdir::WalkDir;

pub fn main() -> iced::Result {
    // Start the GUI.
    FolsumGui::run(Settings::default())
}

#[derive(Debug)]
struct FolsumGui {
    // Track the number of times that each file extension is seen.
    extension_counts: Arc<RwLock<HashMap<String, u32>>>,
}

#[derive(Debug, Clone)]
pub enum GUIMessage {
    StartSummarizing,
    UpdateCounts(WorkerEvent),
    StopSummarizing,
}

impl Application for FolsumGui {
    type Message = GUIMessage;
    type Theme = Theme;
    type Executor = executor::Default;
    // Data needed to initialize the GUI Application.
    type Flags = ();

    fn new(_flags: ()) -> (FolsumGui, Command<GUIMessage>) {
        (
            FolsumGui {
                extension_counts: Arc::new(RwLock::new(HashMap::new())),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        println!("title called");
        // Set the window title to FolSum.
        String::from("FolSum")
    }

    fn update(&mut self, message: GUIMessage) -> Command<GUIMessage> {
        println!("update called");
        match message {
            // If the user wants to start summarizing...
            GUIMessage::StartSummarizing => {
            }
            GUIMessage::StopSummarizing => {
                println!("update: message: StopSummarizing");
            }
            GUIMessage::UpdateCounts(WorkerEvent) => {
                println!("update: message: UpdateCounts");
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<GUIMessage> {
        println!("subscription called");
        // Start the worker thread and interpret update events as count update triggers.
        some_worker(&self.extension_counts).map(GUIMessage::UpdateCounts)
    }

    fn view(&self) -> Element<GUIMessage> {
        println!("view called");
        let unlocked_exts = self.extension_counts.read().unwrap();
        // Alphabetize file extensions before occurrence sorting so those with the same count appear alphabetically.
        let mut ext_info: Vec<(&String, &u32)> = unlocked_exts.iter().sorted().collect();
        // Sort file extensions from most to least occurrences, assuming the user wants to see the most numerous filetypes first.
        ext_info.sort_by(|a, b| b.1.cmp(a.1));

        let table_rows = scrollable(
            Column::with_children(
                ext_info.iter()
                        .map(|(extension_name, times_seen)| text(format!("{extension_name}: {times_seen}")))
                        .map(Element::from)
                        .collect()
            )
        );

        let summarize_button = button("Summarize").on_press(GUIMessage::StartSummarizing);

        let control_pane = Column::new().push(summarize_button);

        let window_content = Row::new()
            .push(control_pane)
            .push(table_rows);

        container(window_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////

// Define the kinds of states that the worker thread can be in.
enum WorkerState {
    Starting,
    ReceiverReadyForMessages(mpsc::Receiver<WorkerInput>),
}

// Define the kinds of processing events that can occur.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    // Channel is spawned and sending a "DoWork" WorkerInput will start processing.
    SenderReadyForMessages(mpsc::Sender<WorkerInput>),
    // Summarization's complete.
    WorkFinished,
}

// Define the kinds of input events that can occur.
pub enum WorkerInput {
    // Start summarizing.
    StartSummarizing,
    // Stop summarizing.
    StopSummarizing,
}


pub fn some_worker(&extension_counts: &Arc<RwLock<HashMap<String, u32>>>) -> Subscription<WorkerEvent> {
    struct SomeWorker;
    // Reset file extension counts to zero.
    *extension_counts.write().unwrap() = HashMap::new();
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let extension_counts_copy = Arc::clone(&extension_counts);
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let extension_counts_copy = Arc::clone(&extension_counts);
    println!("cloned extension counts copy");

    // Start summarizing the given directory in a new thread.
    channel(std::any::TypeId::of::<SomeWorker>(), 100, |mut output| async move {
        println!("in thread, doing things");
        let default_extension = OsString::from("No extension");
        let mut state = WorkerState::Starting;

        println!("starting worker loop");
        loop {
            match &mut state {
                WorkerState::Starting => {
                    println!("worker loop: Starting");
                    // Create channel
                    let (sender, receiver) = mpsc::channel(100);

                    // Send the sender back to the application
                    output.send(WorkerEvent::SenderReadyForMessages(sender)).await;

                    // We are ready to receive messages
                    state = WorkerState::ReceiverReadyForMessages(receiver);
                }
                WorkerState::ReceiverReadyForMessages(receiver) => {
                    println!("worker loop: Ready");
                    // Read next input sent from `Application`
                    let input = receiver.select_next_some().await;

                    match input {
                        WorkerInput::StartSummarizing => {
                            println!("worker loop: WorkerInput::StartSummarizing");
                            // Do some async work...
                            println!("reset extension counts to zero");
                            // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                            for entry_result in WalkDir::new(current_dir().unwrap()).min_depth(1).into_iter() {
                                match entry_result {
                                    Ok(entry) => {
                                        if !entry.file_type().is_dir() {
                                            //println!("Processing file: {:?}", entry.path());
                                            // Extract the file extension from the file's name.
                                            let file_ext: &OsStr = entry.path().extension().unwrap_or(&default_extension);
                                            let show_ext: String = String::from(file_ext.to_string_lossy());
                                            // Lock the extension counts variable so we can add a file to it.
                                            let mut unlocked_counts_copy = extension_counts_copy.write().unwrap();
                                            // Add newly encountered file extensions to known file extensions with a counter of 0.
                                            let counter: &mut u32 = unlocked_counts_copy.entry(show_ext).or_insert(0);
                                            // Increment the counter for known file extensions by one.
                                            *counter += 1;
                                            // Drop the lock on extension counts so the GUI thread can read it.
                                            drop(unlocked_counts_copy);
                                            //println!("Added file");
                                        }; //for loop ends
                                    }
                                    Err(e) => {
                                        println!("Error processing file: {:?}", e)
                                    }
                                }
                            }
                            // Finally, we can optionally produce a message to tell the
                            // `Application` the work is done
                            output.send(WorkerEvent::WorkFinished).await;
                        }
                        WorkerInput::StopSummarizing => {
                        }
                    }
                }
            }
        }
    })
}