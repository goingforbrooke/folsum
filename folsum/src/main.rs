// Standard library libraries.
use std::collections::HashMap;
use std::env::current_dir;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::sync::{Arc, RwLock};

// Iced GUI libraries.
use iced::executor;
use iced::futures::channel::mpsc::Sender;
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
    sender: Option<Sender<WorkerInput>>,
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
                // Initialize the application with no worker thread sender because the thread isn't alive yet.
                sender: None,
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
                println!("update: received message: StartSummarizing");
                let _ = self.sender.as_mut().expect("Worker thread sender isn't initialized yet").send(WorkerInput::StartWork);
            }
            GUIMessage::StopSummarizing => {
                println!("update: message: StopSummarizing");
            }
            GUIMessage::UpdateCounts(worker_event) => match worker_event {
                // If the worker thread has launched and is ready for control messages...
                WorkerEvent::SenderReadyForMessages(sender) => {
                    // ... then make the sending side of the summarization thread available to other parts of the GUI.
                    self.sender = Some(sender);
                    println!("update: worker thread is ready. Sender half is now available for use")
                }
                // If the worker thread has finished summarizing all file extensions...
                WorkerEvent::WorkFinished => {
                    println!("update: received message: WorkFinished")
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<GUIMessage> {
        println!("subscription called");
        // Start the worker thread and interpret update events as count update triggers.
        summarize_directory(&self.extension_counts).map(GUIMessage::UpdateCounts)
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

#[derive(Debug)]
pub enum WorkerInput {
    StartWork,
    StopWork,
}

// Define the kinds of processing events that can be emitted by the worker thread.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    // Channel is spawned and sending a "DoWork" GUIMessage will start processing.
    SenderReadyForMessages(mpsc::Sender<WorkerInput>),
    // Summarization's complete.
    WorkFinished,
}

pub fn summarize_directory(extension_counts: &Arc<RwLock<HashMap<String, u32>>>) -> Subscription<WorkerEvent> {
    struct SomeWorker;
    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
    let extension_counts_copy = extension_counts.clone();
    println!("cloned extension counts copy for thread");
    // Start summarizing the given directory in a new thread.
    channel(std::any::TypeId::of::<SomeWorker>(), 100, move |mut output| { 
        let mut worker_state = WorkerState::Starting;
        // Copy the Arcs of persistent members so they can be accessed by a separate thread.
        let extension_counts_copy = extension_counts_copy.clone();
        // Reset file extension counts to zero.
        *extension_counts_copy.write().unwrap() = HashMap::new();
        println!("cloned extension counts copy for async");
        async move {
            println!("in thread, doing things");
            let default_extension = OsString::from("No extension");

            println!("starting worker loop");
            loop {
                println!("top of loop");
                match &mut worker_state {
                    WorkerState::Starting => {
                        println!("worker loop: Starting");
                        // Create channel
                        let (sender, receiver) = mpsc::channel(100);

                        // Send the sender back to the application
                        let _ = output.send(WorkerEvent::SenderReadyForMessages(sender)).await;

                        // We are ready to receive messages
                        worker_state = WorkerState::ReceiverReadyForMessages(receiver);
                        println!("set worker loop worker_state to ReadyForMessages");
                    }
                    WorkerState::ReceiverReadyForMessages(receiver) => {
                        println!("worker loop worker_state: ReadyForMessages");
                        // Read next input sent from `Application`
                        let input = receiver.select_next_some().await;
                        println!("INPUT");
                        println!("{:?}", input);

                        match input {
                            WorkerInput::StartWork => {
                                println!("worker loop: GUIMessage::StartSummarizing");
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
                                                // Write-lock the extension counts variable so we can add a file to it.
                                                let mut unlocked_counts_copy = extension_counts_copy.write().unwrap();
                                                // Add newly encountered file extensions to known file extensions with a counter of 0.
                                                let counter: &mut u32 = unlocked_counts_copy.entry(show_ext).or_insert(0);
                                                // Increment the counter for known file extensions by one.
                                                *counter += 1;
                                                println!("Added file");
                                            }; //for loop ends
                                        }
                                        Err(e) => {
                                            println!("Error processing file: {:?}", e)
                                        }
                                    }
                                }
                                // Finally, we can optionally produce a message to tell the
                                // `Application` the work is done
                                let _  = output.send(WorkerEvent::WorkFinished).await;
                            }
                            WorkerInput::StopWork => {
                                println!("worker loop: GUIMessage::StopSummarizing(_)")
                            }
                        }
                    }
                }
            }
        }
    })
}