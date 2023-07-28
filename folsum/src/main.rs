// Standard library libraries.
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};

// Iced GUI libraries.
use iced::executor;
use iced::widget::{button, container, text, Column, Row, scrollable};
use iced::{Application, Command, Element, Length, Settings, Theme};


use itertools::Itertools;

// Third-party libraries.
use walkdir::WalkDir;

// Local modules.
mod download;

pub fn main() -> iced::Result {
    FolsumGui::run(Settings::default())
}

#[derive(Debug)]
struct FolsumGui {
    extension_counts: Arc<Mutex<HashMap<String, u32>>>,
    state: SummarizationState,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartSummarizing,
}

impl Application for FolsumGui {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (FolsumGui, Command<Message>) {
        (
            FolsumGui {
                extension_counts: Arc::new(Mutex::new(HashMap::new())),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("FolSum")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::StartSummarizing => {
                // Reset file extension counts to zero.
                *self.extension_counts.lock().unwrap() = HashMap::new();
                // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                let extension_counts_copy = Arc::clone(&self.extension_counts);
                let default_extension = OsString::from("No extension");
                // Recursively iterate through each subdirectory and don't add subdirectories to the result.
                for entry in WalkDir::new(env::current_dir().unwrap())
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
                    // Update the summarization time stopwatch.
                }
            }
        };

        Command::none()
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

        let summarize_button = button("Summarize").on_press(Message::StartSummarizing);

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
