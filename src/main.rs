#![allow(clippy::new_without_default)]
use iced::Application;
pub mod bridge;
mod ui;

fn main() {
    ui::Editor::run(iced::settings::Settings::default());
}
