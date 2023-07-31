// Standard library libraries.
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Iced GUI libraries.
use iced::executor;
use iced::widget::{button, container, text, Column, Row, scrollable};
use iced::{Application, Command, Element, Length, Settings, Theme, time, Subscription};

use dirs::home_dir;


use itertools::Itertools;

// Third-party libraries.
use walkdir::WalkDir;

// Local modules.
//mod download;

pub fn main() -> iced::Result {
    // Start the GUI.
    FolsumGui::run(Settings::default())
}

#[derive(Debug)]
struct FolsumGui {
    // Track the number of times that each file extension is seen.
    extension_counts: Arc<Mutex<HashMap<String, u32>>>,
    // Keep track of whether a directory's being summarized.
    state: SummarizationState,
}

#[derive(Debug)]
pub enum SummarizationState {
    Idle,
    Summarizing,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartSummarizing,
    DisplayCounts(Instant),
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
                extension_counts: Arc::new(Mutex::new(HashMap::new())),
                state: SummarizationState::Idle,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        // Set the window title to FolSum.
        String::from("FolSum")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartSummarizing => match self.state {
                SummarizationState::Idle => {
                    self.state = SummarizationState::Summarizing;
                    // Reset file extension counts to zero.
                    *self.extension_counts.lock().unwrap() = HashMap::new();
                    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                    let extension_counts_copy = Arc::clone(&self.extension_counts);
                    thread::spawn(move || {
                        let default_extension = OsString::from("No extension");
                        // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                        for entry in WalkDir::new(home_dir().unwrap())
                            .min_depth(1)
                            .into_iter()
                            .filter_map(Result::ok)
                            .filter(|e| !e.file_type().is_dir())
                        {
                            // Extract the file extension from the file's name.
                            let file_ext: &OsStr = entry.path().extension().unwrap_or(&default_extension);
                            let show_ext: String = String::from(file_ext.to_string_lossy());
                            // Lock the extension counts variable so we can add a file to it.
                            let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
                            // Add newly encountered file extensions to known file extensions with a counter of 0.
                            let counter: &mut u32 = unlocked_counts_copy.entry(show_ext).or_insert(0);
                            // Increment the counter for known file extensions by one.
                            *counter += 1;
                            drop(unlocked_counts_copy);
                        }
                    });
                }
                SummarizationState::Summarizing => {
                    // Reset file extension counts to zero.
                    *self.extension_counts.lock().unwrap() = HashMap::new();
                    // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                    let extension_counts_copy = Arc::clone(&self.extension_counts);
                    let default_extension = OsString::from("No extension");
                    for entry in WalkDir::new(home_dir().unwrap())
                        .min_depth(1)
                        .into_iter()
                        .filter_map(Result::ok)
                        .filter(|e| !e.file_type().is_dir())
                    {
                        // Extract the file extension from the file's name.
                        let file_ext: &OsStr = entry.path().extension().unwrap_or(&default_extension);
                        let show_ext: String = String::from(file_ext.to_string_lossy());
                        // Lock the extension counts variable so we can add a file to it.
                        let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
                        // Add newly encountered file extensions to known file extensions with a counter of 0.
                        let counter: &mut u32 = unlocked_counts_copy.entry(show_ext).or_insert(0);
                        // Increment the counter for known file extensions by one.
                        *counter += 1;
                        drop(unlocked_counts_copy);
                    }
                }
            }
            Message::DisplayCounts(now) => {
                // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                let extension_counts_copy = Arc::clone(&self.extension_counts);
                // Lock the extension counts variable so we can read it.
                //let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            // If nothing's being summarized, then there are no summarization updates to subscribe to.
            SummarizationState::Idle => Subscription::none(),
            // If a directory's being summarized, then wait ten milliseconds before doing anything, produce a message that file extension counts have been updated, then wait ten milliseconds.
            SummarizationState::Summarizing { .. } => {
                time::every(Duration::from_millis(10)).map(Message::DisplayCounts)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let unlocked_exts = self.extension_counts.lock().unwrap();
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

        let summarize_button = match self.state {
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
