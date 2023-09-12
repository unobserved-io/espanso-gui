mod app;
mod egui_data;
mod espanso_yaml;
mod parse_config;
mod style;

use app::EGUI;
use iced::{window, Application, Settings};

pub fn main() -> iced::Result {
    EGUI::run(Settings {
        window: window::Settings {
            size: (1024, 768),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
