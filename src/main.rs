// espansoGUI - GUI to interface with Espanso
// Copyright (C) 2023 Ricky Kresslein <ricky@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
#![windows_subsystem = "windows"]

mod app;
mod egui_data;
mod espanso_yaml;
mod parse_config;
mod style;

use app::EGUI;
use iced::{window, Application, Settings, Size};

pub fn main() -> iced::Result {
    EGUI::run(Settings {
        window: window::Settings {
            size: Size {
                height: 768.0,
                width: 1024.0,
            },
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
