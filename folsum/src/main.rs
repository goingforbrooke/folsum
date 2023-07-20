use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use iced::executor;
use iced::widget::{button, column, container, progress_bar, scrollable, text, Column};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    Theme,
};

use itertools::Itertools;

mod download;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Debug)]
struct Example {
    downloads: Vec<Download>,
    last_id: usize,
    extension_counts: Arc<Mutex<HashMap<String, u32>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Add,
    Download(usize),
    DownloadProgressed((usize, download::Progress)),
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Example, Command<Message>) {
        (
            Example {
                downloads: vec![Download::new(0)],
                last_id: 0,
                extension_counts: Arc::new(Mutex::new(HashMap::new())),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Download progress - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Add => {
                self.last_id += 1;

                self.downloads.push(Download::new(self.last_id));
            }
            Message::Download(index) => {
                if let Some(download) = self.downloads.get_mut(index) {
                    download.start();
                }
            }
            Message::DownloadProgressed((id, progress)) => {
                if let Some(download) =
                    self.downloads.iter_mut().find(|download| download.id == id)
                {
                    download.progress(progress);
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.downloads.iter().map(Download::subscription))
    }

    fn view(&self) -> Element<Message> {
        let downloads = Column::with_children(
            self.downloads.iter().map(Download::view).collect(),
        )
        .push(
            button("Add another download")
                .on_press(Message::Add)
                .padding(10),
        )
        .spacing(20)
        .align_items(Alignment::End);

        let unlocked_exts = self.extension_counts.lock().unwrap();
        // Alphabetize file extensions before occurrence sorting so those with the same count appear alphabetically.
        let mut ext_info: Vec<(&String, &u32)> = unlocked_exts.iter().sorted().collect();
        // Sort file extensions from most to least occurrences, assuming the user wants to see the most numerous filetypes first.
        ext_info.sort_by(|a, b| b.1.cmp(a.1));

        let mut row_contents = Vec::new();
        for (extension_name, times_seen) in ext_info.iter() {
            row_contents.push(format!("{extension_name}: {times_seen}"));
        }

        let table_rows = scrollable(
            column![
                text(format!("row one")),
                text(format!("row two")),
                text(format!("row three")),
                text(format!("row four")),
                text(format!("row five"))
            ]
        );


        let content = column![downloads, table_rows];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
    }
}

#[derive(Debug)]
struct Download {
    id: usize,
    state: State,
}

#[derive(Debug)]
enum State {
    Idle,
    Downloading { progress: f32 },
    Finished,
    Errored,
}

impl Download {
    pub fn new(id: usize) -> Self {
        Download {
            id,
            state: State::Idle,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle { .. }
            | State::Finished { .. }
            | State::Errored { .. } => {
                self.state = State::Downloading { progress: 0.0 };
            }
            _ => {}
        }
    }

    pub fn progress(&mut self, new_progress: download::Progress) {
        if let State::Downloading { progress } = &mut self.state {
            match new_progress {
                download::Progress::Started => {
                    *progress = 0.0;
                }
                download::Progress::Advanced(percentage) => {
                    *progress = percentage;
                }
                download::Progress::Finished => {
                    self.state = State::Finished;
                }
                download::Progress::Errored => {
                    self.state = State::Errored;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Downloading { .. } => {
                download::file(self.id, "https://speed.hetzner.de/100MB.bin?")
                    .map(Message::DownloadProgressed)
            }
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let current_progress = match &self.state {
            State::Idle { .. } => 0.0,
            State::Downloading { progress } => *progress,
            State::Finished { .. } => 100.0,
            State::Errored { .. } => 0.0,
        };

        let progress_bar = progress_bar(0.0..=100.0, current_progress);

        let control: Element<_> = match &self.state {
            State::Idle => button("Start the download!")
                .on_press(Message::Download(self.id))
                .into(),
            State::Finished => {
                column!["Download finished!", button("Start again")]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .into()
            }
            State::Downloading { .. } => {
                text(format!("Downloading... {current_progress:.2}%")).into()
            }
            State::Errored => column![
                "Something went wrong :(",
                button("Try again").on_press(Message::Download(self.id)),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .into(),
        };

        Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .push(progress_bar)
            .push(control)
            .into()
    }
}
