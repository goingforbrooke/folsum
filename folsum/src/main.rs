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

#[derive(Debug, Clone)]
pub enum Message {
    StartSummarizing,
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
            Message::StartSummarizing => {
                // Reset file extension counts to zero.
                *self.extension_counts.lock().unwrap() = HashMap::new();
                // Copy the Arcs of persistent members so they can be accessed by a separate thread.
                let extension_counts_copy = Arc::clone(&self.extension_counts);
                // Lock the extension counts variable so we can read it.
                let mut unlocked_counts_copy = extension_counts_copy.lock().unwrap();
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
