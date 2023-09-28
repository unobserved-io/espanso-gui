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
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, toggler, tooltip,
    Button, Column, Container, Scrollable, Space, Tooltip,
};
use iced::{alignment, font, Alignment, Application, Command, Element, Length, Renderer};
use iced_aw::graphics::icons;
use iced_aw::{modal, number_input, Card, Icon};
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

#[derive(Debug)]
pub enum EGUI {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
pub struct State {
    espanso_loc: String,
    selected_nav: String,
    directory_invalid: bool,
    selected_file: PathBuf,
    original_file: EspansoYaml,
    edited_file: EspansoYaml,
    original_config: ParsedConfig,
    edited_config: ParsedConfig,
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

impl State {
    fn new() -> Self {
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
            State {
                espanso_loc: egui_data.espanso_dir.clone(),
                selected_nav: "eg-Settings".to_string(),
                directory_invalid: false,
                selected_file: PathBuf::new(),
                original_file: EspansoYaml::default(),
                edited_file: EspansoYaml::default(),
                match_files: {
                    let default_path = PathBuf::from(egui_data.espanso_dir.clone());
                    get_all_match_file_stems(default_path.join("match"))
                },
                original_config: ParsedConfig::default(),
                edited_config: ParsedConfig::default(),
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
            State {
                espanso_loc: String::new(),
                selected_nav: "eg-Settings".to_string(),
                directory_invalid: false,
                selected_file: PathBuf::new(),
                original_file: EspansoYaml::default(),
                edited_file: EspansoYaml::default(),
                original_config: ParsedConfig::default(),
                edited_config: ParsedConfig::default(),
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
    Loaded(Result<(), String>),
    FontLoaded(Result<(), font::Error>),
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

impl Application for EGUI {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            EGUI::Loading,
            Command::batch(vec![
                font::load(iced_aw::graphics::icons::ICON_FONT_BYTES).map(Message::FontLoaded),
                Command::perform(load(), Message::Loaded),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("espansoGUI")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            EGUI::Loading => {
                if let Message::Loaded(_) = message {
                    *self = EGUI::Loaded(State::new())
                }
            }
            EGUI::Loaded(state) => match message {
                Message::ShowModal(title, description, destination) => {
                    state.modal_title = title;
                    state.modal_description = description;
                    state.nav_queue = destination;
                    state.show_modal = true;
                }
                Message::ModalOkPressed => {
                    state.show_modal = false;
                    if state.nav_queue == "eg-Delete" {
                        // Delete state.selected_file
                        match remove_file(state.selected_file.clone()) {
                            Ok(_) => {}
                            Err(err) => eprintln!("Failed to delete file: {}", err),
                        }
                        // Update file list
                        state.match_files = get_all_match_file_stems(
                            PathBuf::from(state.espanso_loc.clone()).join("match"),
                        );
                        // Navigate back to Settings
                        state.nav_queue = String::new();
                        state.modal_ok_text = "OK".to_string();
                        let _ = self.update(Message::NavigateTo("eg-Settings".to_string()));
                    } else if !state.nav_queue.is_empty() {
                        let destination = state.nav_queue.clone();
                        state.nav_queue = String::new();
                        let _ = self.update(Message::NavigateTo(destination));
                    }
                }
                Message::CloseModal => state.show_modal = false,
                Message::ModalCancelPressed => {
                    state.show_modal = false;
                    state.modal_ok_text = "OK".to_string();
                    state.nav_queue = String::new();
                }
                Message::AddPairPressed => {
                    state.edited_file.matches.push(YamlPairs::default());
                    return scrollable::snap_to(
                        SCROLLABLE_ID.clone(),
                        scrollable::RelativeOffset::END,
                    );
                }
                Message::EspansoDirInputChanged(value) => {
                    state.espanso_loc = value;
                }
                Message::YamlInputChanged(new_str, i, trig_repl) => {
                    if trig_repl == "trigger" {
                        state.edited_file.matches.get_mut(i).unwrap().trigger = new_str;
                    } else {
                        state.edited_file.matches.get_mut(i).unwrap().replace = new_str;
                    }
                }
                Message::NavigateTo(value) => {
                    state.selected_nav = value.clone();
                    let espanso_loc = state.espanso_loc.clone();
                    // Reset files to defaults
                    state.original_file = EspansoYaml::default();
                    state.edited_file = EspansoYaml::default();

                    match value.as_str() {
                        "eg-Config" => {
                            state.selected_file =
                                PathBuf::from(espanso_loc + "/config/default.yml");
                            match ParsedConfig::load(&state.selected_file) {
                                Ok(config) => {
                                    state.original_config = config;
                                    // Set combo list prefs to default if not set to prevent it
                                    // loooking like changes were made when they weren't
                                    if state.original_config.backend == None {
                                        state.original_config.backend = Some("Auto".to_string());
                                    }
                                    if state.original_config.toggle_key == None {
                                        state.original_config.toggle_key = Some("OFF".to_string());
                                    }

                                    state.edited_config = state.original_config.clone();
                                }
                                Err(e) => eprintln!("Error {:?}", e),
                            }
                        }
                        "eg-Settings" => state.selected_file = PathBuf::new(),
                        "eg-About" => state.selected_file = PathBuf::new(),
                        _ => {
                            state.selected_file = PathBuf::from(
                                espanso_loc + "/match/" + &state.selected_nav + ".yml",
                            );
                            state.original_file = read_to_triggers(state.selected_file.clone());
                            // for of_match in state.original_file.matches.clone() {
                            //     if !of_match.trigger.is_empty() && !of_match.replace.is_empty() {}
                            // }
                            state.edited_file = state.original_file.clone();
                            state.file_name_change = state.selected_nav.clone();
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
                            state.directory_invalid = false;
                            state.espanso_loc = espanso_dir.into_os_string().into_string().unwrap();
                        } else {
                            state.directory_invalid = true;
                        }
                    }
                }
                Message::SettingsSavePressed => {
                    if state.espanso_loc.ends_with("/") {
                        state.espanso_loc = state.espanso_loc.trim_end_matches("/").to_string();
                    }
                    if valid_espanso_dir(state.espanso_loc.clone()) {
                        state.directory_invalid = false;
                        let new_egui_data = EGUIData {
                            espanso_dir: state.espanso_loc.clone(),
                        };
                        let _ = write_egui_data(&new_egui_data);
                        state.match_files = get_all_match_file_stems(
                            PathBuf::from(state.espanso_loc.clone()).join("match"),
                        )
                    } else {
                        state.directory_invalid = true;
                    }
                }
                Message::ResetPressed => {
                    state.edited_file = state.original_file.clone();
                }
                Message::SaveFilePressed => {
                    let mut empty_lines = false;
                    for pairs in state.edited_file.matches.clone() {
                        if pairs.trigger.trim().is_empty() || pairs.replace.trim().is_empty() {
                            empty_lines = true;
                            break;
                        }
                    }
                    if empty_lines {
                        state.modal_title = "Empty Lines".to_string();
                        state.modal_description = "No text boxes can be empty.".to_string();
                        if !state.nav_queue.is_empty() {
                            state.nav_queue = String::new();
                        }
                        state.show_modal = true;
                    } else {
                        write_from_triggers(state.selected_file.clone(), state.edited_file.clone());
                        state.original_file = state.edited_file.clone();
                    }
                }
                Message::AddFilePressed => {
                    if state.show_new_file_input {
                        state.show_new_file_input = false;
                        state.new_file_name = String::new();
                    } else {
                        state.show_new_file_input = true;
                    }
                }
                Message::NewFileInputChanged(value) => state.new_file_name = value,
                Message::SubmitNewFileName => {
                    state.show_new_file_input = false;
                    if !state.new_file_name.trim().is_empty() {
                        if state.new_file_name.ends_with(".yml") {
                            state.new_file_name =
                                state.new_file_name.trim_end_matches(".yml").to_string()
                        }
                        create_new_yml_file(PathBuf::from(
                            state.espanso_loc.clone() + "/match/" + &state.new_file_name + ".yml",
                        ));
                        state.match_files = get_all_match_file_stems(
                            PathBuf::from(state.espanso_loc.clone()).join("match"),
                        );
                        state.new_file_name = String::new();
                    }
                }
                Message::FileNameChangeInputChanged(value) => {
                    if is_valid_file_name(&value.clone()) {
                        state.file_name_change = value;
                    }
                }
                Message::FileNameChangeSubmit => {
                    if state.file_name_change != state.selected_nav
                        && is_valid_file_name(&state.file_name_change)
                    {
                        let match_path = PathBuf::from(state.espanso_loc.clone()).join("match");
                        let from_path = match_path.join(format!("{}.yml", state.selected_nav));
                        let to_path = match_path.join(format!("{}.yml", state.file_name_change));
                        match rename(from_path, to_path.clone()) {
                            Ok(_) => {}
                            Err(err) => eprintln!("Failed to rename file: {}", err),
                        }

                        // Refresh file list
                        state.match_files = get_all_match_file_stems(match_path);

                        // Set necessary variables to new name
                        state.selected_nav = state.file_name_change.clone();
                        state.selected_file = to_path;
                    }
                }
                Message::DeleteFilePressed => {
                    state.modal_title = "Delete file?".to_string();
                    state.modal_description =
                        "Are you sure you want to delete the file? This cannot be undone."
                            .to_string();
                    state.modal_ok_text = "Delete".to_string();
                    state.nav_queue = "eg-Delete".to_string();
                    state.show_modal = true;
                }
                Message::BackendPicked(value) => state.edited_config.backend = Some(value),
                Message::EnableToggled(value) => state.edited_config.enable = Some(value),
                Message::ToggleKeyPicked(value) => state.edited_config.toggle_key = Some(value),
                Message::InjectDelayInput(value) => state.edited_config.inject_delay = Some(value),
                Message::KeyDelayInput(value) => state.edited_config.key_delay = Some(value),
                Message::ClipboardThresholdInput(value) => {
                    state.edited_config.clipboard_threshold = Some(value)
                }
                Message::PasteShortcutInput(value) => {
                    state.edited_config.paste_shortcut = Some(value)
                }
                Message::SearchShortcutInput(value) => {
                    state.edited_config.search_shortcut = Some(value)
                }
                Message::SearchTriggerInput(value) => {
                    state.edited_config.search_trigger = Some(value)
                }
                Message::PrePasteDelayInput(value) => {
                    state.edited_config.pre_paste_delay = Some(value)
                }
                Message::X11FastInjectToggled(value) => {
                    state.edited_config.disable_x11_fast_inject = Some(value)
                }
                Message::PasteShortcutEventDelayInput(value) => {
                    state.edited_config.paste_shortcut_event_delay = Some(value)
                }
                Message::AutoRestartToggled(value) => {
                    state.edited_config.auto_restart = Some(value)
                }
                Message::PreserveClipboardToggled(value) => {
                    state.edited_config.preserve_clipboard = Some(value)
                }
                Message::RestoreClipboardDelayInput(value) => {
                    state.edited_config.restore_clipboard_delay = Some(value)
                }
                Message::EvdevModifierDelayInput(value) => {
                    state.edited_config.evdev_modifier_delay = Some(value)
                }
                Message::WordSeparatorsInput(value) => {
                    state.edited_config.word_separators =
                        Some(serde_json::from_str(&value).unwrap())
                }
                Message::BackspaceLimitInput(value) => {
                    state.edited_config.backspace_limit = Some(value)
                }
                Message::ApplyPatchToggled(value) => state.edited_config.apply_patch = Some(value),
                Message::KeyboardLayoutInput(value) => {
                    let json_string = format!("{{ \"layout\": \"{}\" }}", value);
                    let map: BTreeMap<String, String> = serde_json::from_str(&json_string).unwrap();
                    state.edited_config.keyboard_layout = Some(map);
                }
                Message::UndoBackspaceToggled(value) => {
                    state.edited_config.undo_backspace = Some(value)
                }
                Message::ShowNotificationsToggled(value) => {
                    state.edited_config.show_notifications = Some(value)
                }
                Message::ShowIconToggled(value) => state.edited_config.show_icon = Some(value),
                Message::UseXclipBackendToggled(value) => {
                    state.edited_config.x11_use_xclip_backend = Some(value)
                }
                Message::ExcludeOrphanEventsToggled(value) => {
                    state.edited_config.win32_exclude_orphan_events = Some(value)
                }
                Message::KeyboardLayoutCacheIntervalInput(value) => {
                    state.edited_config.win32_keyboard_layout_cache_interval = Some(value)
                }
                Message::SaveConfigPressed => {
                    overwrite_config(&state.selected_file.clone(), &state.edited_config.clone());
                    state.original_config = state.edited_config.clone();
                }
                Message::ResetConfigPressed => {
                    state.edited_config = ParsedConfig::default();
                    // Reset combo list prefs to default to prevent it
                    // loooking like changes were made when they weren't
                    state.edited_config.backend = Some("Auto".to_string());
                    state.edited_config.toggle_key = Some("OFF".to_string());
                }
                Message::UndoConfigPressed => state.edited_config = state.original_config.clone(),
                Message::LaunchURL(value) => open_link(&value),
                Message::DeleteRowPressed(index) => {
                    state.edited_file.matches.remove(index);
                }
                _ => {}
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let nav_col = column![
            text("Files").size(20),
            column![text("JA.yaml").size(16)].padding([0, 0, 0, 10]),
            text("Config").size(20),
            text("Settings").size(20)
        ]
        .spacing(12)
        .padding(20)
        .align_items(Alignment::Start);

        match self {
            EGUI::Loading => container(row![
                nav_col,
                column![
                    Space::new(Length::Fill, Length::Fill),
                    text("Loading...")
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .size(50),
                    Space::new(Length::Fill, Length::Fill),
                ]
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            EGUI::Loaded(State {
                espanso_loc,
                selected_nav,
                directory_invalid,
                original_file,
                edited_file,
                original_config,
                edited_config,
                match_files,
                show_modal,
                modal_title,
                modal_description,
                modal_ok_text,
                show_new_file_input,
                new_file_name,
                file_name_change,
                ..
            }) => {
                let unsaved_changes = edited_file.matches != original_file.matches;
                let mut nav_col = column![row![
                    text("Files").size(20),
                    Tooltip::new(
                        button(if show_new_file_input.clone() {
                            "x"
                        } else {
                            "+"
                        })
                        .on_press(Message::AddFilePressed)
                        .style(theme::Button::Text),
                        if show_new_file_input.clone() {
                            "Cancel"
                        } else {
                            "Add a new file"
                        },
                        tooltip::Position::Right,
                    )
                ]
                .spacing(10)
                .align_items(Alignment::Center)]
                .spacing(12)
                .padding(20)
                .width(175)
                .align_items(Alignment::Start);
                let mut yml_files_col: Column<'_, Message, Renderer> =
                    Column::new().spacing(8).padding([0, 0, 0, 10]);
                for yml_file in match_files {
                    yml_files_col =
                        yml_files_col.push(nav_button(yml_file, yml_file, unsaved_changes));
                }
                if show_new_file_input.clone() {
                    yml_files_col = yml_files_col.push(
                        text_input("", new_file_name)
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
                    row![text("Settings").size(25)].padding([0, 0, 20, 0]),
                    column![
                        row![
                            text("Location").size(20),
                            Space::new(10, 0),
                            text_input("", espanso_loc)
                                .on_input(Message::EspansoDirInputChanged)
                                .size(20),
                            Space::new(10, 0),
                            button("Browse").on_press(Message::BrowsePressed),
                        ]
                        .align_items(Alignment::Center),
                        text(if *directory_invalid {
                            "Not a valid directory"
                        } else {
                            ""
                        }),
                    ]
                    .spacing(15)
                    .padding([0, 0, 0, 20]),
                    Space::new(Length::Fill, Length::Fill),
                    row![
                        Space::new(Length::Fill, 0),
                        button("Save").on_press(Message::SettingsSavePressed)
                    ],
                ]
                .padding([20, 20, 20, 20])
                .width(Length::Fill)
                .align_items(Alignment::Start);

                // -- FILE SECTION --
                let mut all_trigger_replace_rows: Column<'_, Message, Renderer> =
                    Column::new().spacing(8).padding([0, 0, 0, 10]);
                if !selected_nav.is_empty()
                    && selected_nav != "eg-Settings"
                    && selected_nav != "eg-Config"
                {
                    all_trigger_replace_rows = all_trigger_replace_rows.push(
                        row![
                            button("+ Add").on_press(Message::AddPairPressed),
                            text(format!("Items: {}", original_file.matches.len())),
                            Space::new(Length::Fill, 0),
                            text_input(file_name_change, file_name_change)
                                .on_input(Message::FileNameChangeInputChanged)
                                .on_submit(Message::FileNameChangeSubmit),
                            text(if file_name_change != selected_nav {
                                "Press enter to save changes"
                            } else {
                                ""
                            }),
                            Space::new(Length::Fill, 0),
                            button(text(Icon::Trash).font(icons::ICON_FONT))
                                .on_press(Message::DeleteFilePressed)
                                .style(theme::Button::Destructive),
                            button("Reset").on_press_maybe(
                                match original_file.matches == edited_file.matches {
                                    true => None,
                                    false => Some(Message::ResetPressed),
                                }
                            ),
                            button("Save").on_press_maybe(
                                match original_file.matches == edited_file.matches {
                                    true => None,
                                    false => Some(Message::SaveFilePressed),
                                }
                            ),
                        ]
                        .align_items(Alignment::Center)
                        .spacing(10),
                    );

                    for i in 0..edited_file.matches.len() {
                        all_trigger_replace_rows = all_trigger_replace_rows.push(
                            Container::new(
                                row![
                                    button(text(Icon::Trash).font(icons::ICON_FONT))
                                        .on_press(Message::DeleteRowPressed(i))
                                        .style(theme::Button::Text),
                                    column![
                                        row![
                                            text("Trigger:").size(20).width(90),
                                            text_input(
                                                &edited_file.matches[i].trigger,
                                                &edited_file.matches[i].trigger
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
                                        .align_items(Alignment::Center),
                                        row![
                                            text("Replace:").size(20).width(90),
                                            text_input(
                                                &edited_file.matches[i].replace,
                                                &edited_file.matches[i].replace
                                            )
                                            .on_input(move |new_string| {
                                                Message::YamlInputChanged(
                                                    new_string,
                                                    i,
                                                    "replace".to_string(),
                                                )
                                            })
                                            .size(20)
                                        ]
                                        .align_items(Alignment::Center)
                                    ]
                                    .spacing(8),
                                ]
                                .padding(20)
                                .align_items(Alignment::Center)
                                .spacing(12),
                            )
                            .style(style::gray_background),
                        );
                    }
                }

                let open_file_col =
                    column![
                        Scrollable::new(all_trigger_replace_rows.padding([20, 20, 20, 40]))
                            .id(SCROLLABLE_ID.clone())
                    ]
                    .width(Length::Fill)
                    .align_items(Alignment::Start);

                // -- CONFIG SECTION --
                let paste_shortcut = if edited_config.paste_shortcut.is_some() {
                    edited_config.paste_shortcut.clone().unwrap()
                } else {
                    if env::consts::OS == "macos" {
                        "CMD+V".to_string()
                    } else {
                        "CTRL+V".to_string()
                    }
                };
                let search_shortcut = if edited_config.search_shortcut.is_some() {
                    edited_config.search_shortcut.clone().unwrap()
                } else {
                    "ALT+SPACE".to_string()
                };
                let search_trigger = if edited_config.search_trigger.is_some() {
                    edited_config.search_trigger.clone().unwrap()
                } else {
                    "off".to_string()
                };
                let word_separators = if edited_config.word_separators.is_some() {
                    serde_json::to_string(&edited_config.word_separators.clone().unwrap())
                        .unwrap_or_default()
                } else {
                    "[\" \", \",\", \".\", \"?\", \"!\", \"\\r\", \"\\n\", 22]".to_string()
                };
                let keyboard_layout = if edited_config.keyboard_layout.is_some() {
                    if edited_config
                        .keyboard_layout
                        .clone()
                        .unwrap()
                        .contains_key("layout")
                    {
                        edited_config.keyboard_layout.clone().unwrap()["layout"].clone()
                    } else {
                        "us".to_string()
                    }
                } else {
                    "us".to_string()
                };

                let all_config_rows = column!(
                    row![
                        Tooltip::new(
                            button(text(Icon::Trash).font(icons::ICON_FONT))
                                .on_press(Message::ResetConfigPressed)
                                .style(theme::Button::Destructive),
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
                            .style(theme::Button::Secondary),
                        Space::new(Length::Fill, 0),
                        Tooltip::new(
                            button(text(Icon::ArrowCounterclockwise).font(icons::ICON_FONT))
                                .on_press_maybe(match original_config == edited_config {
                                    true => None,
                                    false => Some(Message::UndoConfigPressed),
                                })
                                .style(theme::Button::Secondary),
                            if original_config != edited_config {
                                "Undo unsaved changes"
                            } else {
                                ""
                            },
                            tooltip::Position::Bottom,
                        ),
                        button("Save").on_press_maybe(match original_config == edited_config {
                            true => None,
                            false => Some(Message::SaveConfigPressed),
                        }),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10)
                    .padding([0, 0, 20, 0]),
                    row![
                        text("Backend").size(20).width(300),
                        pick_list(
                            vec![
                                "Auto".to_string(),
                                "Clipboard".to_string(),
                                "Inject".to_string(),
                            ],
                            if edited_config.backend.clone().unwrap_or_default().is_empty() {
                                Some("auto".to_string())
                            } else {
                                edited_config.backend.clone()
                            },
                            Message::BackendPicked
                        )
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Enable").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.enable.is_some() {
                                edited_config.enable.clone().unwrap()
                            } else {
                                true
                            },
                            Message::EnableToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
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
                            edited_config.toggle_key.clone(),
                            Message::ToggleKeyPicked
                        )
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Inject delay").size(20).width(300),
                        number_input(
                            if edited_config.inject_delay.is_some() {
                                edited_config.inject_delay.unwrap()
                            } else {
                                0
                            },
                            1000,
                            Message::InjectDelayInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Key delay").size(20).width(300),
                        number_input(
                            if edited_config.key_delay.is_some() {
                                edited_config.key_delay.unwrap()
                            } else {
                                0
                            },
                            1000,
                            Message::KeyDelayInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Clipboard threshold").size(20).width(300),
                        number_input(
                            if edited_config.clipboard_threshold.is_some() {
                                edited_config.clipboard_threshold.unwrap()
                            } else {
                                100
                            },
                            1000,
                            Message::ClipboardThresholdInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
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
                    .align_items(Alignment::Center),
                    row![
                        text("Search shortcut").size(20).width(300),
                        text_input("ALT+SPACE", &search_shortcut)
                            .on_input(Message::SearchShortcutInput)
                            .width(Length::Fixed(130.0))
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Search trigger").size(20).width(300),
                        text_input("off", &search_trigger)
                            .on_input(Message::SearchTriggerInput)
                            .width(Length::Fixed(130.0))
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Pre-paste delay").size(20).width(300),
                        number_input(
                            if edited_config.pre_paste_delay.is_some() {
                                edited_config.pre_paste_delay.unwrap()
                            } else {
                                300
                            },
                            1000,
                            Message::PrePasteDelayInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Disable X11 fast inject").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.disable_x11_fast_inject.is_some() {
                                edited_config.disable_x11_fast_inject.clone().unwrap()
                            } else {
                                false
                            },
                            Message::X11FastInjectToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Paste shortcut event delay").size(20).width(300),
                        number_input(
                            if edited_config.paste_shortcut_event_delay.is_some() {
                                edited_config.paste_shortcut_event_delay.unwrap()
                            } else {
                                10
                            },
                            1000,
                            Message::PasteShortcutEventDelayInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Auto restart").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.auto_restart.is_some() {
                                edited_config.auto_restart.clone().unwrap()
                            } else {
                                true
                            },
                            Message::AutoRestartToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Preserve clipboard").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.preserve_clipboard.is_some() {
                                edited_config.preserve_clipboard.clone().unwrap()
                            } else {
                                true
                            },
                            Message::PreserveClipboardToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Restore clipboard delay").size(20).width(300),
                        number_input(
                            if edited_config.restore_clipboard_delay.is_some() {
                                edited_config.restore_clipboard_delay.unwrap()
                            } else {
                                300
                            },
                            1000,
                            Message::RestoreClipboardDelayInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("EVDEV modifier delay").size(20).width(300),
                        number_input(
                            if edited_config.evdev_modifier_delay.is_some() {
                                edited_config.evdev_modifier_delay.unwrap()
                            } else {
                                10
                            },
                            1000,
                            Message::EvdevModifierDelayInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Word separators").size(20).width(300),
                        text_input(
                            "[\" \", \",\", \".\", \"?\", \"!\", \"\\r\", \"\\n\", 22]",
                            &word_separators
                        )
                        .on_input(Message::WordSeparatorsInput)
                        .width(Length::Fixed(130.0))
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Backspace limit").size(20).width(300),
                        number_input(
                            if edited_config.backspace_limit.is_some() {
                                edited_config.backspace_limit.unwrap()
                            } else {
                                5
                            },
                            100,
                            Message::BackspaceLimitInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Apply patch").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.apply_patch.is_some() {
                                edited_config.apply_patch.clone().unwrap()
                            } else {
                                true
                            },
                            Message::ApplyPatchToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Keyboard layout").size(20).width(300),
                        text_input("us", &keyboard_layout)
                            .on_input(Message::KeyboardLayoutInput)
                            .width(Length::Fixed(130.0))
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Undo backspace").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.undo_backspace.is_some() {
                                edited_config.undo_backspace.clone().unwrap()
                            } else {
                                true
                            },
                            Message::UndoBackspaceToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Show notifications").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.show_notifications.is_some() {
                                edited_config.show_notifications.clone().unwrap()
                            } else {
                                true
                            },
                            Message::ShowNotificationsToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Show icon").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.show_icon.is_some() {
                                edited_config.show_icon.clone().unwrap()
                            } else {
                                true
                            },
                            Message::ShowIconToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("X11 use xclip backend").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.x11_use_xclip_backend.is_some() {
                                edited_config.x11_use_xclip_backend.clone().unwrap()
                            } else {
                                false
                            },
                            Message::UseXclipBackendToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Win32 exclude orphan events").size(20).width(300),
                        toggler(
                            "".to_string(),
                            if edited_config.win32_exclude_orphan_events.is_some() {
                                edited_config.win32_exclude_orphan_events.clone().unwrap()
                            } else {
                                true
                            },
                            Message::ExcludeOrphanEventsToggled
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        text("Win32 keyboard layout cache interval")
                            .size(20)
                            .width(300),
                        number_input(
                            if edited_config.win32_keyboard_layout_cache_interval.is_some() {
                                edited_config.win32_keyboard_layout_cache_interval.unwrap()
                            } else {
                                2000
                            },
                            10000,
                            Message::KeyboardLayoutCacheIntervalInput
                        )
                        .width(Length::Shrink)
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                )
                .spacing(8)
                .padding([0, 0, 0, 10]);

                let config_col =
                    column![Scrollable::new(all_config_rows.padding([20, 20, 20, 40]))
                        .id(SCROLLABLE_ID.clone())]
                    .width(Length::Fill)
                    .align_items(Alignment::Start);

                // -- ABOUT SECTION --
                let about_col = column![
                    row![text("About").size(25)].padding([20, 0, 20, 20]),
                    column![
                        Space::new(Length::Fill, 0),
                        row![
                            button("espanso")
                                .on_press(Message::LaunchURL("https://espanso.org".to_string()))
                                .style(theme::Button::Secondary),
                            text("was created by").size(20),
                            button("Federico Terzi")
                                .on_press(Message::LaunchURL(
                                    "https://federicoterzi.com".to_string()
                                ))
                                .style(theme::Button::Secondary)
                        ]
                        .spacing(5),
                        row![
                            text("espansoGUI was created by").size(20),
                            button("Ricky Kresslein")
                                .on_press(Message::LaunchURL("https://kressle.in".to_string()))
                                .style(theme::Button::Secondary)
                        ]
                        .spacing(5),
                        row![
                            text("It was built with").size(20),
                            button("Rust")
                                .on_press(Message::LaunchURL("https://www.rust-lang.org/".to_string()))
                                .style(theme::Button::Secondary),
                            text("and").size(20),
                            button("Iced")
                                .on_press(Message::LaunchURL("https://github.com/iced-rs/iced".to_string()))
                                .style(theme::Button::Secondary),
                        ]
                        .spacing(5),
                        row![
                            text("espansoGUI is under active development and may not be perfect. Please backup your espanso directory before using this program to modify any files.")
                        ].padding([0,40,0,40]),
                    ]
                    .spacing(15)
                    .align_items(Alignment::Center),
                    // row![text("Known Issues").size(20)].padding([0, 0, 0, 20]),
                    // column![
                    //     text("- ").size(18),
                    // ].padding([0, 0, 0, 20]),
                    row![text("Upcoming Features").size(20)].padding([0, 0, 0, 20]),
                    column![
                        text("- Ability to search YAML files").size(18),
                        text("- Ability to create backups of the espanso directory").size(18),
                    ].padding([0, 0, 0, 20]),
                ].spacing(15);

                let main_row = row![
                    nav_col,
                    match selected_nav.as_str() {
                        "eg-Settings" => settings_col,
                        "eg-Config" => config_col,
                        "eg-About" => about_col,
                        _ => open_file_col,
                    }
                ];

                let underlay = Container::new(main_row)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill);

                let overlay: Option<Card<'_, Message, Renderer>> = if show_modal.clone() {
                    Some(
                        Card::new(text(modal_title), text(modal_description))
                            .foot(
                                row![
                                    button(
                                        text("Cancel")
                                            .horizontal_alignment(alignment::Horizontal::Center)
                                    )
                                    .width(Length::Fill)
                                    .on_press(Message::ModalCancelPressed),
                                    button(
                                        text(modal_ok_text)
                                            .horizontal_alignment(alignment::Horizontal::Center)
                                    )
                                    .width(Length::Fill)
                                    .style(if modal_ok_text == "Delete" {
                                        theme::Button::Destructive
                                    } else {
                                        theme::Button::Primary
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

                modal(underlay, overlay)
                    .backdrop(Message::CloseModal)
                    .on_esc(Message::CloseModal)
                    .into()
            }
        }
    }
}

async fn load() -> Result<(), String> {
    Ok(())
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
        .style(theme::Button::Text)
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
