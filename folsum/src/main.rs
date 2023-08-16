// Standard library libraries.
use std::collections::HashMap;
use std::env::current_dir;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::sync::{Arc, RwLock};
use std::thread;

// Iced GUI libraries.
use iced::executor;
use iced::widget::{button, container, text, Column, Row, scrollable};
use iced::{Application, Command, Element, Length, Settings, Theme, Subscription};
use iced::futures::channel::mpsc;

use iced::subscription::{self, channel};
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
    // Keep track of whether a directory's being summarized.
    state: Arc<RwLock<SummarizationState>>,
}

#[derive(Debug)]
pub enum SummarizationState {
    // Starting
    Idle,
    // Ready to receive messages.
    Summarizing,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartSummarizing,
    DisplayCounts(Event),
    StopSummarizing,
}

impl Application for FolsumGui {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    // Data needed to initialize the GUI Application.
    type Flags = ();

    fn new(_flags: ()) -> (FolsumGui, Command<Message>) {
        (
            FolsumGui {
                extension_counts: Arc::new(RwLock::new(HashMap::new())),
                state: Arc::new(RwLock::new(SummarizationState::Idle)),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        println!("title called");
        // Set the window title to FolSum.
        String::from("FolSum")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("update called");
        match message {
            // If the user wants to start summarizing...
            Message::StartSummarizing => match *self.state.read().unwrap() {
                // ... and nothing's being summarized...
                SummarizationState::Idle => {
                    println!("update: Idle: Summarize button clicked");
                    // Reset file extension counts to zero.
                    *self.extension_counts.write().unwrap() = HashMap::new();
                    println!("reset extension counts to zero");
                    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                    let extension_counts_copy = Arc::clone(&self.extension_counts);
                    println!("cloned extension counts copy");
                    // Clone the state Arc so we can update it once Summarization's finished.
                    let state_copy = Arc::clone(&self.state);
                    println!("cloned state copy");
                    // Start summarizing the given directory in a new thread.
                    thread::spawn(move || {
                        {
                            // Set the state to "Summarizing."
                            println!("setting state to summarizing");
                            let mut state_copy = state_copy.write().unwrap();
                            *state_copy = SummarizationState::Summarizing;
                            println!("set state to summarizing");
                        }
                        println!("in thread, doing things");
                        let default_extension = OsString::from("No extension");
                        // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                        for entry_result in WalkDir::new(current_dir().unwrap()).min_depth(1).into_iter() {
                            match entry_result {
                                Ok(entry) => {
                                    if !entry.file_type().is_dir() {
                                        //println!("Processing file: {:?}", entry.path());
                                        { // `this_state` comes into scope
                                            // While summarizing the given directory in a new thread, periodically check...
                                            //println!("before state_copy.read()");
                                            let this_state = state_copy.read().unwrap();
                                            //println!("after state_copy.read()");
                                            match *this_state {
                                                // ... if this summarization should be cancelled...
                                                SummarizationState::Idle => {
                                                    // ... then stop summarizing.
                                                    println!("in thread, state is now idle, so stopping");
                                                    return;
                                                }
                                                // ... or if this summarization's still in progress, so work should continue.
                                                SummarizationState::Summarizing => {
                                                    //println!("in thread, state is now summarizing, so continuing work");
                                                    // continue work
                                                }
                                            }
                                        } // `this_state` goes out of scope
                                        //println!("in thread, in for loop, doing old summarizing");
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
                        println!("ended summarization for loop");
                        {
                            // set the state to "Idle."
                            println!("setting state to idle");
                            // Now that summarization's finished, set the state to "idle."
                            let mut state = state_copy.write().unwrap();
                            *state = SummarizationState::Idle;
                            println!("set state to idle");
                        }
                    });
                    println!("thread scope ended");
                }
                SummarizationState::Summarizing => {
                    println!("update: Summarizing while already summarizing");
                }
            }
            Message::DisplayCounts(_) => {
                println!("update: message: displaycounts: Summarize button clicked");
                // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                let extension_counts_copy = Arc::clone(&self.extension_counts);
                // Lock the extension counts variable so we can read it.
                //let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
            }
            Message::StopSummarizing => {
                {
                    println!("update: message: StopSummarizing: Summarize button clicked");
                    // set the state to "Idle."
                    println!("setting state to idle");
                    // Now that summarization's finished, set the state to "idle."
                    let mut state = self.state.write().unwrap();
                    *state = SummarizationState::Idle;
                    println!("set state to idle");
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        println!("subscription called");
        // Start the worker thread and start listening for Input events.
        some_worker().map(Message::DisplayCounts)
    }

    fn view(&self) -> Element<Message> {
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

        let summarize_button = match *self.state.read().unwrap() {
            // If nothing's being summarized...
            SummarizationState::Idle => {
                // ... then make a button that'll start summarization.
                button("Summarize").on_press(Message::StartSummarizing)
            },
            // If something's being summarized...
            SummarizationState::Summarizing => {
                // ... then make a button that'll stop summarization.
                button("Cancel").on_press(Message::StopSummarizing)
            }
        };

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

// Define the kinds of processing events that can occur.
#[derive(Debug, Clone)]
pub enum Event {
    // Channel is spawned and sending a "DoWork" Input will start processing.
    SenderReadyForMessages(mpsc::Sender<Input>),
    // Summarization's complete.
    WorkFinished,
}

// Define the kinds of input events that can occur.
pub enum Input {
    // Start summarizing.
    DoSomeWork,
}

// Define the kinds of states that the worker thread can be in.
enum State {
    Starting,
    ReceiverReadyForMessages(mpsc::Receiver<Input>),
}

pub fn some_worker() -> Subscription<Event> {
    struct SomeWorker;

    channel(std::any::TypeId::of::<SomeWorker>(), 100, |mut output| async move {
        let mut state = State::Starting;

        println!("starting worker loop");
        loop {
            match &mut state {
                State::Starting => {
                    println!("worker loop: Starting");
                    // Create channel
                    let (sender, receiver) = mpsc::channel(100);

                    // Send the sender back to the application
                    output.send(Event::SenderReadyForMessages(sender)).await;

                    // We are ready to receive messages
                    state = State::ReceiverReadyForMessages(receiver);
                }
                State::ReceiverReadyForMessages(receiver) => {
                    println!("worker loop: Ready");
                    // Read next input sent from `Application`
                    let input = receiver.select_next_some().await;

                    match input {
                        Input::DoSomeWork => {
                            println!("worker loop: Input::DoSomeWork");
                            // Do some async work...

                            // Finally, we can optionally produce a message to tell the
                            // `Application` the work is done
                            output.send(Event::WorkFinished).await;
                        }
                    }
                }
            }
        }
    })
}