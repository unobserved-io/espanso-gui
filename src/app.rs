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

use crate::{
    egui_data::EGUIData,
    espanso_yaml::{EspansoYaml, YamlPairs},
    parse_config::ParsedConfig,
    style,
};

use dirs::config_dir;
use home;
use iced::{
    alignment,
    widget::{
        button, center, column, container, horizontal_space, mouse_area, opaque, pick_list, row,
        scrollable, stack, text, text_editor, text_input, toggler, tooltip, Button, Column,
        Container, Scrollable, Space, Theme, Tooltip,
    },
    Alignment, Color, Element, Length, Padding, Renderer, Task,
};
use iced_aw::{number_input, Card};
use iced_fonts::{nerd::icon_to_char, Nerd, NERD_FONT};
use once_cell::sync::Lazy;
use regex::Regex;
use rfd::FileDialog;
use std::collections::BTreeMap;
use std::env;
use std::fs::{create_dir, remove_file, rename, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub struct EGUI {
    espanso_loc: String,
    selected_nav: String,
    directory_invalid: bool,
    selected_file: PathBuf,
    original_file: EspansoYaml,
    edited_file: EspansoYaml,
    edited_file_te: Vec<text_editor::Content>,
    original_config: ParsedConfig,
    edited_config: ParsedConfig,
    temp_word_separators: String,
    match_files: Vec<String>,
    show_modal: bool,
    modal_title: String,
    modal_description: String,
    modal_ok_text: String,
    nav_queue: String,
    show_new_file_input: bool,
    new_file_name: String,
    file_name_change: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddPairPressed,
    EspansoDirInputChanged(String),
    YamlInputChanged(String, usize, String),
    BrowsePressed,
    SettingsSavePressed,
    NavigateTo(String),
    ResetPressed,
    SaveFilePressed,
    ModalCancelPressed,
    ModalOkPressed,
    CloseModal,
    ShowModal(String, String, String),
    EditReplace(text_editor::Action, usize),
    AddFilePressed,
    NewFileInputChanged(String),
    SubmitNewFileName,
    FileNameChangeInputChanged(String),
    FileNameChangeSubmit,
    DeleteFilePressed,
    BackendPicked(String),
    EnableToggled(bool),
    ToggleKeyPicked(String),
    InjectDelayInput(usize),
    KeyDelayInput(usize),
    ClipboardThresholdInput(usize),
    PasteShortcutInput(String),
    SearchShortcutInput(String),
    SearchTriggerInput(String),
    PrePasteDelayInput(usize),
    X11FastInjectToggled(bool),
    PasteShortcutEventDelayInput(usize),
    AutoRestartToggled(bool),
    PreserveClipboardToggled(bool),
    RestoreClipboardDelayInput(usize),
    EvdevModifierDelayInput(usize),
    WordSeparatorsInput(String),
    BackspaceLimitInput(usize),
    ApplyPatchToggled(bool),
    KeyboardLayoutInput(String),
    UndoBackspaceToggled(bool),
    ShowNotificationsToggled(bool),
    ShowIconToggled(bool),
    UseXclipBackendToggled(bool),
    ExcludeOrphanEventsToggled(bool),
    KeyboardLayoutCacheIntervalInput(i64),
    SaveConfigPressed,
    UndoConfigPressed,
    ResetConfigPressed,
    LaunchURL(String),
    DeleteRowPressed(usize),
}

impl Default for EGUI {
    fn default() -> Self {
        Self::new()
    }
}

impl EGUI {
    pub fn new() -> Self {
        let egui_data = match read_egui_data() {
            Ok(data) => data,
            Err(_) => EGUIData {
                espanso_dir: get_default_espanso_dir(),
            },
        };
        if valid_espanso_dir(egui_data.espanso_dir.clone()) {
            let new_egui_data = EGUIData {
                espanso_dir: egui_data.espanso_dir.clone(),
            };
            let _ = write_egui_data(&new_egui_data);
            EGUI {
                espanso_loc: egui_data.espanso_dir.clone(),
                selected_nav: "eg-Settings".to_string(),
                directory_invalid: false,
                selected_file: PathBuf::new(),
                original_file: EspansoYaml::default(),
                edited_file: EspansoYaml::default(),
                edited_file_te: Vec::new(),
                match_files: {
                    let default_path = PathBuf::from(egui_data.espanso_dir.clone());
                    get_all_match_file_stems(default_path.join("match"))
                },
                original_config: ParsedConfig::default(),
                edited_config: ParsedConfig::default(),
                temp_word_separators: String::new(),
                show_modal: false,
                modal_title: String::new(),
                modal_description: String::new(),
                modal_ok_text: "OK".to_string(),
                nav_queue: String::new(),
                show_new_file_input: false,
                new_file_name: String::new(),
                file_name_change: String::new(),
            }
        } else {
            EGUI {
                espanso_loc: String::new(),
                selected_nav: "eg-Settings".to_string(),
                directory_invalid: false,
                selected_file: PathBuf::new(),
                original_file: EspansoYaml::default(),
                edited_file: EspansoYaml::default(),
                edited_file_te: Vec::new(),
                original_config: ParsedConfig::default(),
                edited_config: ParsedConfig::default(),
                temp_word_separators: String::new(),
                match_files: Vec::new(),
                show_modal: false,
                modal_title: String::new(),
                modal_description: String::new(),
                modal_ok_text: "OK".to_string(),
                nav_queue: String::new(),
                show_new_file_input: false,
                new_file_name: String::new(),
                file_name_change: String::new(),
            }
        }
    }

    pub fn title(&self) -> String {
        String::from("espansoGUI")
    }

    pub fn theme(&self) -> Theme {
        match dark_light::detect() {
            dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            dark_light::Mode::Dark => Theme::Dark,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowModal(title, description, destination) => {
                self.modal_title = title;
                self.modal_description = description;
                self.nav_queue = destination;
                self.show_modal = true;
            }
            Message::ModalOkPressed => {
                self.show_modal = false;
                if self.nav_queue == "eg-Delete" {
                    // Delete self.selected_file
                    match remove_file(self.selected_file.clone()) {
                        Ok(_) => {}
                        Err(err) => eprintln!("Failed to delete file: {}", err),
                    }
                    // Update file list
                    self.match_files = get_all_match_file_stems(
                        PathBuf::from(self.espanso_loc.clone()).join("match"),
                    );
                    // Navigate back to Settings
                    self.nav_queue = String::new();
                    self.modal_ok_text = "OK".to_string();
                    let _ = self.update(Message::NavigateTo("eg-Settings".to_string()));
                } else if !self.nav_queue.is_empty() {
                    let destination = self.nav_queue.clone();
                    self.nav_queue = String::new();
                    let _ = self.update(Message::NavigateTo(destination));
                }
            }
            Message::CloseModal => self.show_modal = false,
            Message::ModalCancelPressed => {
                self.show_modal = false;
                self.modal_ok_text = "OK".to_string();
                self.nav_queue = String::new();
            }
            Message::AddPairPressed => {
                self.edited_file.matches.push(YamlPairs::default());
                return scrollable::snap_to(SCROLLABLE_ID.clone(), scrollable::RelativeOffset::END);
            }
            Message::EspansoDirInputChanged(value) => {
                self.espanso_loc = value;
            }
            Message::YamlInputChanged(new_str, i, trig_repl) => {
                if trig_repl == "trigger" {
                    self.edited_file.matches.get_mut(i).unwrap().trigger = new_str;
                } else {
                    self.edited_file.matches.get_mut(i).unwrap().replace = new_str;
                }
            }
            Message::NavigateTo(value) => {
                self.selected_nav = value.clone();
                let espanso_loc = self.espanso_loc.clone();
                // Reset files to defaults
                self.original_file = EspansoYaml::default();
                self.edited_file = EspansoYaml::default();

                match value.as_str() {
                    "eg-Config" => {
                        self.selected_file = PathBuf::from(espanso_loc + "/config/default.yml");
                        match ParsedConfig::load(&self.selected_file) {
                            Ok(config) => {
                                self.original_config = config;
                                // Set combo list prefs to default if not set to prevent it
                                // loooking like changes were made when they weren't
                                if self.original_config.backend == None {
                                    self.original_config.backend = Some("Auto".to_string());
                                }
                                if self.original_config.toggle_key == None {
                                    self.original_config.toggle_key = Some("OFF".to_string());
                                }

                                self.edited_config = self.original_config.clone();
                                self.temp_word_separators =
                                    if self.edited_config.word_separators.is_some() {
                                        serde_json::to_string(
                                            &self.edited_config.word_separators.clone().unwrap(),
                                        )
                                        .unwrap_or_default()
                                    } else {
                                        format!("{:?}", get_default_word_separators())
                                    };
                            }
                            Err(e) => eprintln!("Error {:?}", e),
                        }
                    }
                    "eg-Settings" => self.selected_file = PathBuf::new(),
                    "eg-About" => self.selected_file = PathBuf::new(),
                    _ => {
                        self.selected_file =
                            PathBuf::from(espanso_loc + "/match/" + &self.selected_nav + ".yml");
                        self.original_file = read_to_triggers(self.selected_file.clone());
                        self.edited_file = self.original_file.clone();
                        // copy matches to text_editor
                        self.edited_file_te.clear();
                        for a_match in self.edited_file.matches.clone() {
                            self.edited_file_te
                                .push(text_editor::Content::with_text(&a_match.replace));
                        }
                        self.file_name_change = self.selected_nav.clone();
                    }
                }
            }
            Message::BrowsePressed => {
                let default_path_mac: PathBuf = ["Library", "Application Support", "espanso"]
                    .iter()
                    .collect();
                let mut default_espanso_path = PathBuf::new();
                match home::home_dir() {
                    Some(path) => {
                        default_espanso_path = path;
                        default_espanso_path = default_espanso_path.join(default_path_mac);
                    }
                    None => println!("User directory not found"),
                }
                let selected_folder = FileDialog::new()
                    .set_directory(default_espanso_path)
                    .pick_folder();

                if selected_folder.is_some() {
                    let espanso_dir = selected_folder.unwrap();
                    if valid_espanso_dir(espanso_dir.display().to_string()) {
                        self.directory_invalid = false;
                        self.espanso_loc = espanso_dir.into_os_string().into_string().unwrap();
                    } else {
                        self.directory_invalid = true;
                    }
                }
            }
            Message::SettingsSavePressed => {
                if self.espanso_loc.ends_with("/") {
                    self.espanso_loc = self.espanso_loc.trim_end_matches("/").to_string();
                }
                if valid_espanso_dir(self.espanso_loc.clone()) {
                    self.directory_invalid = false;
                    let new_egui_data = EGUIData {
                        espanso_dir: self.espanso_loc.clone(),
                    };
                    let _ = write_egui_data(&new_egui_data);
                    self.match_files = get_all_match_file_stems(
                        PathBuf::from(self.espanso_loc.clone()).join("match"),
                    )
                } else {
                    self.directory_invalid = true;
                }
            }
            Message::ResetPressed => {
                self.edited_file = self.original_file.clone();
                self.edited_file_te.clear();
                for a_match in self.edited_file.matches.clone() {
                    self.edited_file_te
                        .push(text_editor::Content::with_text(&a_match.replace));
                }
            }
            Message::SaveFilePressed => {
                let mut empty_lines = false;
                for pairs in self.edited_file.matches.clone() {
                    if pairs.trigger.trim().is_empty() || pairs.replace.trim().is_empty() {
                        empty_lines = true;
                        break;
                    }
                }
                if empty_lines {
                    self.modal_title = "Empty Lines".to_string();
                    self.modal_description = "No text boxes can be empty.".to_string();
                    if !self.nav_queue.is_empty() {
                        self.nav_queue = String::new();
                    }
                    self.show_modal = true;
                } else {
                    write_from_triggers(self.selected_file.clone(), self.edited_file.clone());
                    self.original_file = self.edited_file.clone();
                }
            }
            Message::AddFilePressed => {
                if self.show_new_file_input {
                    self.show_new_file_input = false;
                    self.new_file_name = String::new();
                } else {
                    self.show_new_file_input = true;
                }
            }
            Message::NewFileInputChanged(value) => self.new_file_name = value,
            Message::SubmitNewFileName => {
                self.show_new_file_input = false;
                if !self.new_file_name.trim().is_empty() {
                    if self.new_file_name.ends_with(".yml") {
                        self.new_file_name = self.new_file_name.trim_end_matches(".yml").to_string()
                    }
                    create_new_yml_file(PathBuf::from(
                        self.espanso_loc.clone() + "/match/" + &self.new_file_name + ".yml",
                    ));
                    self.match_files = get_all_match_file_stems(
                        PathBuf::from(self.espanso_loc.clone()).join("match"),
                    );
                    self.new_file_name = String::new();
                }
            }
            Message::FileNameChangeInputChanged(value) => {
                if is_valid_file_name(&value.clone()) {
                    self.file_name_change = value;
                }
            }
            Message::FileNameChangeSubmit => {
                if self.file_name_change != self.selected_nav
                    && is_valid_file_name(&self.file_name_change)
                {
                    let match_path = PathBuf::from(self.espanso_loc.clone()).join("match");
                    let from_path = match_path.join(format!("{}.yml", self.selected_nav));
                    let to_path = match_path.join(format!("{}.yml", self.file_name_change));
                    match rename(from_path, to_path.clone()) {
                        Ok(_) => {}
                        Err(err) => eprintln!("Failed to rename file: {}", err),
                    }

                    // Refresh file list
                    self.match_files = get_all_match_file_stems(match_path);

                    // Set necessary variables to new name
                    self.selected_nav = self.file_name_change.clone();
                    self.selected_file = to_path;
                }
            }
            Message::DeleteFilePressed => {
                self.modal_title = "Delete file?".to_string();
                self.modal_description =
                    "Are you sure you want to delete the file? This cannot be undone.".to_string();
                self.modal_ok_text = "Delete".to_string();
                self.nav_queue = "eg-Delete".to_string();
                self.show_modal = true;
            }
            Message::BackendPicked(value) => self.edited_config.backend = Some(value),
            Message::EnableToggled(value) => self.edited_config.enable = Some(value),
            Message::ToggleKeyPicked(value) => self.edited_config.toggle_key = Some(value),
            Message::InjectDelayInput(value) => self.edited_config.inject_delay = Some(value),
            Message::KeyDelayInput(value) => self.edited_config.key_delay = Some(value),
            Message::ClipboardThresholdInput(value) => {
                self.edited_config.clipboard_threshold = Some(value)
            }
            Message::PasteShortcutInput(value) => self.edited_config.paste_shortcut = Some(value),
            Message::SearchShortcutInput(value) => self.edited_config.search_shortcut = Some(value),
            Message::SearchTriggerInput(value) => self.edited_config.search_trigger = Some(value),
            Message::PrePasteDelayInput(value) => self.edited_config.pre_paste_delay = Some(value),
            Message::X11FastInjectToggled(value) => {
                self.edited_config.disable_x11_fast_inject = Some(value)
            }
            Message::PasteShortcutEventDelayInput(value) => {
                self.edited_config.paste_shortcut_event_delay = Some(value)
            }
            Message::AutoRestartToggled(value) => self.edited_config.auto_restart = Some(value),
            Message::PreserveClipboardToggled(value) => {
                self.edited_config.preserve_clipboard = Some(value)
            }
            Message::RestoreClipboardDelayInput(value) => {
                self.edited_config.restore_clipboard_delay = Some(value)
            }
            Message::EvdevModifierDelayInput(value) => {
                self.edited_config.evdev_modifier_delay = Some(value)
            }
            Message::WordSeparatorsInput(value) => {
                self.temp_word_separators = value;
            }
            Message::BackspaceLimitInput(value) => self.edited_config.backspace_limit = Some(value),
            Message::ApplyPatchToggled(value) => self.edited_config.apply_patch = Some(value),
            Message::KeyboardLayoutInput(value) => {
                let json_string = format!("{{ \"layout\": \"{}\" }}", value);
                let map: BTreeMap<String, String> = serde_json::from_str(&json_string).unwrap();
                self.edited_config.keyboard_layout = Some(map);
            }
            Message::UndoBackspaceToggled(value) => self.edited_config.undo_backspace = Some(value),
            Message::ShowNotificationsToggled(value) => {
                self.edited_config.show_notifications = Some(value)
            }
            Message::ShowIconToggled(value) => self.edited_config.show_icon = Some(value),
            Message::UseXclipBackendToggled(value) => {
                self.edited_config.x11_use_xclip_backend = Some(value)
            }
            Message::ExcludeOrphanEventsToggled(value) => {
                self.edited_config.win32_exclude_orphan_events = Some(value)
            }
            Message::KeyboardLayoutCacheIntervalInput(value) => {
                self.edited_config.win32_keyboard_layout_cache_interval = Some(value)
            }
            Message::SaveConfigPressed => {
                let word_separators_changed = self.temp_word_separators.to_owned()
                    != if self.edited_config.word_separators.is_some() {
                        serde_json::to_string(&self.edited_config.word_separators.clone().unwrap())
                            .unwrap_or_default()
                    } else {
                        format!("{:?}", get_default_word_separators())
                    };
                if word_separators_changed {
                    let mut corrected_string = self.temp_word_separators.clone();
                    if !corrected_string.contains("\\\\r") {
                        corrected_string = corrected_string.replace("\\r", "\\\\r");
                    }

                    if !corrected_string.contains("\\\\n") {
                        corrected_string = corrected_string.replace("\\n", "\\\\n");
                    }

                    if !corrected_string.contains("\\\\u0016") {
                        corrected_string = corrected_string.replace("\\u{16}", "\\\\u0016");
                    }

                    match serde_json::from_str::<Vec<String>>(&corrected_string) {
                        Ok(value) => {
                            self.edited_config.word_separators = Some(value);
                        }
                        Err(err) => eprintln!("Couldn't parse WS: {}", err),
                    };
                }

                overwrite_config(&self.selected_file.clone(), &self.edited_config.clone());
                self.original_config = self.edited_config.clone();
                self.temp_word_separators = if self.edited_config.word_separators.is_some() {
                    serde_json::to_string(&self.edited_config.word_separators.clone().unwrap())
                        .unwrap_or_default()
                } else {
                    format!("{:?}", get_default_word_separators())
                };
            }
            Message::ResetConfigPressed => {
                self.edited_config = ParsedConfig::default();
                self.temp_word_separators = if self.edited_config.word_separators.is_some() {
                    serde_json::to_string(&self.edited_config.word_separators.clone().unwrap())
                        .unwrap_or_default()
                } else {
                    format!("{:?}", get_default_word_separators())
                };
                // Reset combo list prefs to default to prevent it
                // loooking like changes were made when they weren't
                self.edited_config.backend = Some("Auto".to_string());
                self.edited_config.toggle_key = Some("OFF".to_string());
            }
            Message::UndoConfigPressed => {
                self.edited_config = self.original_config.clone();
                self.temp_word_separators = if self.edited_config.word_separators.is_some() {
                    serde_json::to_string(&self.edited_config.word_separators.clone().unwrap())
                        .unwrap_or_default()
                } else {
                    format!("{:?}", get_default_word_separators())
                };
            }
            Message::LaunchURL(value) => open_link(&value),
            Message::DeleteRowPressed(index) => {
                self.edited_file.matches.remove(index);
            }
            Message::EditReplace(action, i) => match action {
                text_editor::Action::Scroll { lines: _ } => {}
                action => {
                    let is_edit = action.is_edit();
                    self.edited_file_te[i].perform(action);

                    if is_edit {
                        match self.edited_file.matches.get_mut(i) {
                            Some(s) => {
                                s.replace = self.edited_file_te[i]
                                    .text()
                                    .trim_end_matches('\n')
                                    .to_string()
                            }
                            None => eprintln!("No matching string for trigger"),
                        }
                    }
                }
            },
        }

        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let unsaved_changes = self.edited_file.matches != self.original_file.matches;
        let word_separators_changed = self.temp_word_separators.to_owned()
            != if self.edited_config.word_separators.is_some() {
                serde_json::to_string(&self.edited_config.word_separators.clone().unwrap())
                    .unwrap_or_default()
            } else {
                format!("{:?}", get_default_word_separators())
            };
        let mut nav_col = column![row![
            text("Files").size(20),
            Tooltip::new(
                button(if self.show_new_file_input.clone() {
                    "x"
                } else {
                    "+"
                })
                .on_press(Message::AddFilePressed)
                .style(button::text),
                if self.show_new_file_input.clone() {
                    "Cancel"
                } else {
                    "Add a new file"
                },
                tooltip::Position::Right,
            )
        ]
        .spacing(10)
        .align_y(Alignment::Center)]
        .spacing(12)
        .padding(20)
        .width(175)
        .align_x(Alignment::Start);
        let mut yml_files_col: Column<'_, Message, Theme, Renderer> =
            Column::new().spacing(8).padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 10.0,
            });
        for yml_file in &self.match_files {
            yml_files_col = yml_files_col.push(nav_button(yml_file, yml_file, unsaved_changes));
        }
        if self.show_new_file_input.clone() {
            yml_files_col = yml_files_col.push(
                text_input("", &self.new_file_name)
                    .on_input(Message::NewFileInputChanged)
                    .on_submit(Message::SubmitNewFileName),
            )
        }
        nav_col = nav_col.push(yml_files_col);
        nav_col = nav_col.push(nav_button("Config", "eg-Config", unsaved_changes));
        nav_col = nav_col.push(nav_button("Settings", "eg-Settings", unsaved_changes));
        nav_col = nav_col.push(nav_button("About", "eg-About", false));

        // -- SETTINGS SECTION --
        let settings_col = column![
            row![text("Settings").size(25)].padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 20.0,
                left: 0.0,
            }),
            column![
                row![
                    text("Location").size(20),
                    Space::new(10, 0),
                    text_input("", &self.espanso_loc)
                        .on_input(Message::EspansoDirInputChanged)
                        .size(20),
                    Space::new(10, 0),
                    button("Browse").on_press(Message::BrowsePressed),
                ]
                .align_y(Alignment::Center),
                text(if self.directory_invalid {
                    "Not a valid directory"
                } else {
                    ""
                }),
            ]
            .spacing(15)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 20.0,
            }),
            Space::new(Length::Fill, Length::Fill),
            row![
                Space::new(Length::Fill, 0),
                button("Save").on_press(Message::SettingsSavePressed)
            ],
        ]
        .padding(20)
        .width(Length::Fill)
        .align_x(Alignment::Start);

        // -- FILE SECTION --
        let mut all_trigger_replace_rows: Column<'_, Message, Theme, Renderer> =
            Column::new().spacing(8).padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 10.0,
            });
        if !self.selected_nav.is_empty()
            && self.selected_nav != "eg-Settings"
            && self.selected_nav != "eg-Config"
        {
            all_trigger_replace_rows = all_trigger_replace_rows.push(
                row![
                    button("+ Add").on_press(Message::AddPairPressed),
                    text(format!("Items: {}", self.original_file.matches.len())),
                    Space::new(Length::Fill, 0),
                    text_input(&self.file_name_change, &self.file_name_change)
                        .on_input(Message::FileNameChangeInputChanged)
                        .on_submit(Message::FileNameChangeSubmit),
                    text(if self.file_name_change != self.selected_nav {
                        "Press enter to save changes"
                    } else {
                        ""
                    }),
                    Space::new(Length::Fill, 0),
                    button(text(icon_to_char(Nerd::TrashOne)).font(NERD_FONT))
                        .on_press(Message::DeleteFilePressed)
                        .style(button::danger),
                    button("Reset").on_press_maybe(
                        match self.original_file.matches == self.edited_file.matches {
                            true => None,
                            false => Some(Message::ResetPressed),
                        }
                    ),
                    button("Save").on_press_maybe(
                        match self.original_file.matches == self.edited_file.matches {
                            true => None,
                            false => Some(Message::SaveFilePressed),
                        }
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(10),
            );

            for i in 0..self.edited_file.matches.len() {
                all_trigger_replace_rows = all_trigger_replace_rows.push(
                    Container::new(
                        row![
                            button(text(icon_to_char(Nerd::TrashOne)).font(NERD_FONT))
                                .on_press(Message::DeleteRowPressed(i))
                                .style(button::text),
                            column![
                                row![
                                    text("Trigger:").size(20).width(90),
                                    text_input(
                                        &self.edited_file.matches[i].trigger,
                                        &self.edited_file.matches[i].trigger
                                    )
                                    .on_input(move |new_string| {
                                        Message::YamlInputChanged(
                                            new_string,
                                            i,
                                            "trigger".to_string(),
                                        )
                                    })
                                    .size(20)
                                ]
                                .align_y(Alignment::Center),
                                row![
                                    text("Replace:").size(20).width(90),
                                    text_editor(&self.edited_file_te[i]).on_action(move |action| {
                                        Message::EditReplace(action, i)
                                    })
                                ]
                                .align_y(Alignment::Center)
                            ]
                            .spacing(8),
                        ]
                        .padding(20)
                        .align_y(Alignment::Center)
                        .spacing(12),
                    )
                    .style(style::gray_background),
                );
            }
            if self.edited_file.matches.len() > 2 {
                all_trigger_replace_rows = all_trigger_replace_rows.push(
                    row![
                        Space::new(Length::Fill, 0),
                        button("+ Add").on_press(Message::AddPairPressed),
                        button("Save").on_press_maybe(
                            match self.original_file.matches == self.edited_file.matches {
                                true => None,
                                false => Some(Message::SaveFilePressed),
                            },
                        )
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                );
            }
        }

        let open_file_col = column![Scrollable::new(all_trigger_replace_rows.padding(Padding {
            top: 20.0,
            right: 20.0,
            bottom: 20.0,
            left: 40.0,
        }))
        .id(SCROLLABLE_ID.clone())]
        .width(Length::Fill)
        .align_x(Alignment::Start);

        // -- CONFIG SECTION --
        let paste_shortcut = if self.edited_config.paste_shortcut.is_some() {
            self.edited_config.paste_shortcut.clone().unwrap()
        } else {
            if env::consts::OS == "macos" {
                "CMD+V".to_string()
            } else {
                "CTRL+V".to_string()
            }
        };
        let search_shortcut = if self.edited_config.search_shortcut.is_some() {
            self.edited_config.search_shortcut.clone().unwrap()
        } else {
            "ALT+SPACE".to_string()
        };
        let search_trigger = if self.edited_config.search_trigger.is_some() {
            self.edited_config.search_trigger.clone().unwrap()
        } else {
            "off".to_string()
        };
        let word_separators = if !self.temp_word_separators.is_empty() {
            self.temp_word_separators.to_owned()
        } else {
            format!("{:?}", get_default_word_separators())
        };
        let keyboard_layout = if self.edited_config.keyboard_layout.is_some() {
            if self
                .edited_config
                .keyboard_layout
                .clone()
                .unwrap()
                .contains_key("layout")
            {
                self.edited_config.keyboard_layout.clone().unwrap()["layout"].clone()
            } else {
                "us".to_string()
            }
        } else {
            "us".to_string()
        };

        let all_config_rows = column!(
            row![
                Tooltip::new(
                    button(text(icon_to_char(Nerd::TrashOne)).font(NERD_FONT))
                        .on_press(Message::ResetConfigPressed)
                        .style(button::danger),
                    "Reset all to defaults",
                    tooltip::Position::Bottom,
                ),
                Space::new(Length::Fill, 0),
                text("For information on each of these values, please vist"),
                button("espanso.org")
                    .on_press(Message::LaunchURL(
                        "https://espanso.org/docs/configuration/options/#options-reference"
                            .to_string()
                    ))
                    .style(button::secondary),
                Space::new(Length::Fill, 0),
                Tooltip::new(
                    button(text(icon_to_char(Nerd::RotateLeft)).font(NERD_FONT))
                        .on_press_maybe(
                            match self.original_config == self.edited_config
                                && !word_separators_changed
                            {
                                true => None,
                                false => Some(Message::UndoConfigPressed),
                            }
                        )
                        .style(button::secondary),
                    if self.original_config != self.edited_config || word_separators_changed {
                        "Undo unsaved changes"
                    } else {
                        ""
                    },
                    tooltip::Position::Bottom,
                ),
                button("Save").on_press_maybe(
                    match self.original_config == self.edited_config && !word_separators_changed {
                        true => None,
                        false => Some(Message::SaveConfigPressed),
                    }
                ),
            ]
            .align_y(Alignment::Center)
            .spacing(10)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 20.0,
                left: 0.0,
            }),
            row![
                text("Backend").size(20).width(300),
                pick_list(
                    vec![
                        "Auto".to_string(),
                        "Clipboard".to_string(),
                        "Inject".to_string(),
                    ],
                    if self
                        .edited_config
                        .backend
                        .clone()
                        .unwrap_or_default()
                        .is_empty()
                    {
                        Some("auto".to_string())
                    } else {
                        self.edited_config.backend.clone()
                    },
                    Message::BackendPicked
                )
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Enable").size(20).width(300),
                toggler(if self.edited_config.enable.is_some() {
                    self.edited_config.enable.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::EnableToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Toggle key").size(20).width(300),
                pick_list(
                    vec![
                        "OFF".to_string(),
                        "CTRL".to_string(),
                        "ALT".to_string(),
                        "SHIFT".to_string(),
                        "META".to_string(),
                        "LEFT_CTRL".to_string(),
                        "LEFT_ALT".to_string(),
                        "LEFT_SHIFT".to_string(),
                        "LEFT_META".to_string(),
                        "RIGHT_CTRL".to_string(),
                        "RIGHT_ALT".to_string(),
                        "RIGHT_SHIFT".to_string(),
                        "RIGHT_META".to_string(),
                    ],
                    self.edited_config.toggle_key.clone(),
                    Message::ToggleKeyPicked
                )
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Inject delay").size(20).width(300),
                number_input(
                    if self.edited_config.inject_delay.is_some() {
                        self.edited_config.inject_delay.unwrap()
                    } else {
                        0
                    },
                    0..1000,
                    Message::InjectDelayInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Key delay").size(20).width(300),
                number_input(
                    if self.edited_config.key_delay.is_some() {
                        self.edited_config.key_delay.unwrap()
                    } else {
                        0
                    },
                    0..1000,
                    Message::KeyDelayInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Clipboard threshold").size(20).width(300),
                number_input(
                    if self.edited_config.clipboard_threshold.is_some() {
                        self.edited_config.clipboard_threshold.unwrap()
                    } else {
                        100
                    },
                    0..1000,
                    Message::ClipboardThresholdInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Paste shortcut").size(20).width(300),
                text_input(
                    if env::consts::OS == "macos" {
                        "CMD+V"
                    } else {
                        "CTRL+V"
                    },
                    &paste_shortcut,
                )
                .on_input(Message::PasteShortcutInput)
                .width(Length::Fixed(130.0))
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Search shortcut").size(20).width(300),
                text_input("ALT+SPACE", &search_shortcut)
                    .on_input(Message::SearchShortcutInput)
                    .width(Length::Fixed(130.0))
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Search trigger").size(20).width(300),
                text_input("off", &search_trigger)
                    .on_input(Message::SearchTriggerInput)
                    .width(Length::Fixed(130.0))
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Pre-paste delay").size(20).width(300),
                number_input(
                    if self.edited_config.pre_paste_delay.is_some() {
                        self.edited_config.pre_paste_delay.unwrap()
                    } else {
                        300
                    },
                    0..1000,
                    Message::PrePasteDelayInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Disable X11 fast inject").size(20).width(300),
                toggler(if self.edited_config.disable_x11_fast_inject.is_some() {
                    self.edited_config.disable_x11_fast_inject.clone().unwrap()
                } else {
                    false
                })
                .on_toggle(Message::X11FastInjectToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Paste shortcut event delay").size(20).width(300),
                number_input(
                    if self.edited_config.paste_shortcut_event_delay.is_some() {
                        self.edited_config.paste_shortcut_event_delay.unwrap()
                    } else {
                        10
                    },
                    0..1000,
                    Message::PasteShortcutEventDelayInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Auto restart").size(20).width(300),
                toggler(if self.edited_config.auto_restart.is_some() {
                    self.edited_config.auto_restart.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::AutoRestartToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Preserve clipboard").size(20).width(300),
                toggler(if self.edited_config.preserve_clipboard.is_some() {
                    self.edited_config.preserve_clipboard.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::PreserveClipboardToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Restore clipboard delay").size(20).width(300),
                number_input(
                    if self.edited_config.restore_clipboard_delay.is_some() {
                        self.edited_config.restore_clipboard_delay.unwrap()
                    } else {
                        300
                    },
                    0..1000,
                    Message::RestoreClipboardDelayInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("EVDEV modifier delay").size(20).width(300),
                number_input(
                    if self.edited_config.evdev_modifier_delay.is_some() {
                        self.edited_config.evdev_modifier_delay.unwrap()
                    } else {
                        10
                    },
                    0..1000,
                    Message::EvdevModifierDelayInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Word separators").size(20).width(300),
                text_input(
                    &format!("{:?}", get_default_word_separators()),
                    &word_separators
                )
                .on_input(Message::WordSeparatorsInput)
                .width(Length::Fixed(130.0))
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Backspace limit").size(20).width(300),
                number_input(
                    if self.edited_config.backspace_limit.is_some() {
                        self.edited_config.backspace_limit.unwrap()
                    } else {
                        5
                    },
                    0..100,
                    Message::BackspaceLimitInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Apply patch").size(20).width(300),
                toggler(if self.edited_config.apply_patch.is_some() {
                    self.edited_config.apply_patch.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::ApplyPatchToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Keyboard layout").size(20).width(300),
                text_input("us", &keyboard_layout)
                    .on_input(Message::KeyboardLayoutInput)
                    .width(Length::Fixed(130.0))
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Undo backspace").size(20).width(300),
                toggler(if self.edited_config.undo_backspace.is_some() {
                    self.edited_config.undo_backspace.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::UndoBackspaceToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Show notifications").size(20).width(300),
                toggler(if self.edited_config.show_notifications.is_some() {
                    self.edited_config.show_notifications.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::ShowNotificationsToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Show icon").size(20).width(300),
                toggler(if self.edited_config.show_icon.is_some() {
                    self.edited_config.show_icon.clone().unwrap()
                } else {
                    true
                })
                .on_toggle(Message::ShowIconToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("X11 use xclip backend").size(20).width(300),
                toggler(if self.edited_config.x11_use_xclip_backend.is_some() {
                    self.edited_config.x11_use_xclip_backend.clone().unwrap()
                } else {
                    false
                })
                .on_toggle(Message::UseXclipBackendToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Win32 exclude orphan events").size(20).width(300),
                toggler(
                    if self.edited_config.win32_exclude_orphan_events.is_some() {
                        self.edited_config
                            .win32_exclude_orphan_events
                            .clone()
                            .unwrap()
                    } else {
                        true
                    }
                )
                .on_toggle(Message::ExcludeOrphanEventsToggled)
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text("Win32 keyboard layout cache interval")
                    .size(20)
                    .width(300),
                number_input(
                    if self
                        .edited_config
                        .win32_keyboard_layout_cache_interval
                        .is_some()
                    {
                        self.edited_config
                            .win32_keyboard_layout_cache_interval
                            .unwrap()
                    } else {
                        2000
                    },
                    0..10000,
                    Message::KeyboardLayoutCacheIntervalInput
                )
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .spacing(8)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 10.0,
        });

        let config_col = column![Scrollable::new(all_config_rows.padding(Padding {
            top: 20.0,
            right: 20.0,
            bottom: 20.0,
            left: 40.0,
        }))
        .id(SCROLLABLE_ID.clone())]
        .width(Length::Fill)
        .align_x(Alignment::Start);

        // -- ABOUT SECTION --
        let about_col = column![
                    row![text("About").size(25)].padding(Padding {
                                    top: 20.0,
                                    right: 0.0,
                                    bottom: 20.0,
                                    left: 20.0,
                                }),
                    column![
                        Space::new(Length::Fill, 0),
                        row![
                            button("espanso")
                                .on_press(Message::LaunchURL("https://espanso.org".to_string()))
                                .style(button::secondary),
                            text("was created by").size(20),
                            button("Federico Terzi")
                                .on_press(Message::LaunchURL(
                                    "https://federicoterzi.com".to_string()
                                ))
                                .style(button::secondary)
                        ]
                        .spacing(5),
                        row![
                            text("espansoGUI was created by").size(20),
                            button("Ricky Kresslein")
                                .on_press(Message::LaunchURL("https://kressle.in".to_string()))
                                .style(button::secondary)
                        ]
                        .spacing(5),
                        row![
                            text("It was built with").size(20),
                            button("Rust")
                                .on_press(Message::LaunchURL("https://www.rust-lang.org/".to_string()))
                                .style(button::secondary),
                            text("and").size(20),
                            button("Iced")
                                .on_press(Message::LaunchURL("https://github.com/iced-rs/iced".to_string()))
                                .style(button::secondary),
                        ]
                        .spacing(5),
                        row![
                            text("espansoGUI is under active development and may not be perfect. Please backup your espanso directory before using this program to modify any files.")
                        ].padding([0,40]),
                    ]
                    .spacing(15)
                    .align_x(Alignment::Center),
                    row![text("Upcoming Features").size(20)].padding(Padding {
                                    top: 0.0,
                                    right: 0.0,
                                    bottom: 0.0,
                                    left: 20.0,
                                }),
                    column![
                        text("- Ability to search YAML files").size(18),
                        text("- Ability to create backups of the espanso directory").size(18),
                    ].padding(Padding {
                                    top: 0.0,
                                    right: 0.0,
                                    bottom: 0.0,
                                    left: 20.0,
                                }),
                ].spacing(15);

        let main_row = row![
            nav_col,
            match self.selected_nav.as_str() {
                "eg-Settings" => settings_col,
                "eg-Config" => config_col,
                "eg-About" => about_col,
                _ => open_file_col,
            }
        ];

        let underlay = Container::new(main_row)
            .width(Length::Fill)
            .height(Length::Fill);

        let overlay: Option<Card<'_, Message, Theme, Renderer>> = if self.show_modal.clone() {
            Some(
                Card::new(text(&self.modal_title), text(&self.modal_description))
                    .foot(
                        row![
                            button(text("Cancel").align_x(alignment::Horizontal::Center))
                                .width(Length::Fill)
                                .on_press(Message::ModalCancelPressed),
                            button(
                                text(&self.modal_ok_text).align_x(alignment::Horizontal::Center)
                            )
                            .width(Length::Fill)
                            .style(if self.modal_ok_text == "Delete" {
                                button::danger
                            } else {
                                button::primary
                            })
                            .on_press(Message::ModalOkPressed),
                        ]
                        .spacing(10)
                        .padding(5)
                        .width(Length::Fill),
                    )
                    .max_width(300.0)
                    .on_close(Message::CloseModal),
            )
        } else {
            None
        };

        if let Some(alert) = overlay {
            modal(underlay, container(alert), Message::CloseModal).into()
        } else {
            underlay.into()
        }
    }
}

fn get_app_dir() -> PathBuf {
    if let Some(config_dir) = config_dir() {
        // Mac: /Users/username/Library/Application Support/espansoGUI
        return config_dir.join("espansoGUI");
    } else {
        return PathBuf::from("./");
    }
}

fn read_egui_data() -> Result<EGUIData, Box<dyn std::error::Error>> {
    let path_to_file = get_app_dir().join("egui_data.json");
    let mut file = File::open(path_to_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: EGUIData = serde_json::from_str(&contents)?;
    Ok(data)
}

fn write_egui_data(data: &EGUIData) -> Result<(), Box<dyn std::error::Error>> {
    let directory = get_app_dir();
    if !directory.is_dir() {
        match create_dir(directory.clone()) {
            Ok(_) => println!("App directory created successfully."),
            Err(err) => eprintln!("Failed to create directory: {}", err),
        }
    }
    let path_to_file = directory.join("egui_data.json");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path_to_file)?;

    let serialized = serde_json::to_string(data)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

fn read_to_triggers(path: PathBuf) -> EspansoYaml {
    let file = File::open(path.clone()).expect("Could not open file.");
    let yaml: EspansoYaml = serde_yaml::from_reader(file).expect("Could not read values.");
    let filtered_yaml: Vec<YamlPairs> = yaml
        .matches
        .into_iter()
        .filter(|pair| !pair.trigger.is_empty() && !pair.replace.is_empty())
        .collect();
    EspansoYaml {
        matches: filtered_yaml,
    }
}

fn write_from_triggers(path: PathBuf, edited_file: EspansoYaml) {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open file");
    serde_yaml::to_writer(file, &edited_file).unwrap();
}

fn create_new_yml_file(file_path: PathBuf) {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .expect("Couldn't open file");
    serde_yaml::to_writer(file, &EspansoYaml::default()).unwrap();
}

fn overwrite_config(path: &Path, config: &ParsedConfig) {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't write config to file");
    serde_yaml::to_writer(file, config).unwrap();
}

fn get_default_espanso_dir() -> String {
    if let Some(config_dir) = config_dir() {
        let default_path = config_dir.join("espanso");
        return default_path.display().to_string();
    }

    String::new()
}

fn valid_espanso_dir(selected_dir: String) -> bool {
    // Check if expected directories and files exist to verify it is valid
    let selected_dir: PathBuf = PathBuf::from(selected_dir);
    let config_dir = selected_dir.join("config");
    let match_dir = selected_dir.join("match");
    let config_exists: bool = config_dir.is_dir();
    let match_exists: bool = match_dir.is_dir();
    let config_yml_exists: bool = selected_dir.join("config/default.yml").is_file();
    if config_exists && match_exists && config_yml_exists {
        true
    } else {
        false
    }
}

fn get_all_match_file_stems(match_dir: PathBuf) -> Vec<String> {
    let mut match_file_stems = Vec::new();
    // Walk the directory and get all .yml file names
    for entry in WalkDir::new(match_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "yml" {
                    match_file_stems.push(
                        entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
            }
        }
    }

    match_file_stems
}

fn nav_button<'a>(
    text: &'a str,
    destination: &'a str,
    unsaved_changes: bool,
) -> Button<'a, Message> {
    button(text)
        .on_press({
            if unsaved_changes {
                Message::ShowModal(
                    "Unsaved Changes".to_string(),
                    "Leaving this file with erase any unsaved changes.".to_string(),
                    destination.to_string(),
                )
            } else {
                Message::NavigateTo(destination.to_string())
            }
        })
        .style(button::text)
}

fn is_valid_file_name(file_name: &str) -> bool {
    let pattern = Regex::new(r"^[\w\-. ]+$").unwrap();
    pattern.is_match(file_name)
}

fn open_link(url: &str) {
    if let Err(err) = webbrowser::open(url) {
        eprintln!("Failed to open link: {}", err);
    }
}

fn get_default_word_separators() -> Vec<String> {
    vec![
        " ".to_string(),
        ",".to_string(),
        ".".to_string(),
        "?".to_string(),
        "!".to_string(),
        "\r".to_string(),
        "\n".to_string(),
        (22u8 as char).to_string(),
    ]
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    alert: Container<'a, Message>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(
                center(opaque(row![horizontal_space(), alert, horizontal_space()])).style(
                    |_theme| {
                        container::Style {
                            background: Some(
                                Color {
                                    a: 0.8,
                                    ..Color::BLACK
                                }
                                .into(),
                            ),
                            ..container::Style::default()
                        }
                    }
                )
            )
            .on_press(on_blur)
        )
    ]
    .into()
}
