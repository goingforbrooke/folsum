#![warn(clippy::all, rust_2018_idioms)]

mod gui;
pub use gui::FolsumGui;

mod export_csv;
pub use export_csv::export_csv;
