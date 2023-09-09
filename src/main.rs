mod egui_data;
mod espanso_yaml;

use dirs::config_dir;
use egui_data::EGUIData;
use espanso_yaml::{EspansoYaml, YamlPairs};
use home;
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, row, scrollable, text, text_input, tooltip, Button, Column,
    Container, Scrollable, Space, Tooltip,
};
use iced::{
    alignment, font, window, Alignment, Application, Command, Element, Length, Renderer, Settings,
};
use iced_aw::graphics::icons;
use iced_aw::Icon;
use iced_aw::{modal, Card};
use once_cell::sync::Lazy;
use regex::Regex;
use rfd::FileDialog;
use serde_yaml::{from_reader, to_writer};
use std::fs::{create_dir, remove_file, rename, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command as p_cmd;
use walkdir::WalkDir;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub fn main() -> iced::Result {
    EGUI::run(Settings {
        window: window::Settings {
            size: (1024, 768),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug)]
enum EGUI {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
struct State {
    espanso_loc: String,
    selected_nav: String,
    directory_invalid: bool,
    selected_file: PathBuf,
    original_file: EspansoYaml,
    edited_file: EspansoYaml,
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
enum Message {
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
}

impl Application for EGUI {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        // (EGUI::Loaded(State::new()), Command::none())
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
                    match value.as_str() {
                        "eg-Config" => {
                            state.selected_file =
                                PathBuf::from(espanso_loc + "/config/default.yml");
                        }
                        "eg-Settings" => state.selected_file = PathBuf::new(),
                        _ => {
                            state.selected_file = PathBuf::from(
                                espanso_loc + "/match/" + &state.selected_nav + ".yml",
                            );
                            state.original_file = read_to_triggers(state.selected_file.clone());
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
                    write_from_triggers(state.selected_file.clone(), state.edited_file.clone());
                    state.original_file = state.edited_file.clone();
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

                let settings_col = column![
                    row![text("Settings").size(25)].padding([0, 0, 20, 0]),
                    column![
                        text("espanso is not running").size(20),
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
                .padding([20, 20, 20, 40])
                .width(Length::Fill)
                .align_items(Alignment::Start);

                let mut all_trigger_replace_rows: Column<'_, Message, Renderer> =
                    Column::new().spacing(8).padding([0, 0, 0, 10]);
                if !selected_nav.is_empty() && selected_nav != "eg-Settings" {
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
                                row![column![
                                    row![
                                        text("Trigger:").size(20).width(90),
                                        text_input(
                                            &edited_file.matches[i].trigger,
                                            &edited_file.matches[i].trigger
                                        )
                                        .on_input(move |s| {
                                            Message::YamlInputChanged(s, i, "trigger".to_string())
                                        })
                                        .size(20)
                                    ],
                                    row![
                                        text("Replace:").size(20).width(75),
                                        text_input(
                                            &edited_file.matches[i].replace,
                                            &edited_file.matches[i].replace
                                        )
                                        .on_input(move |s| {
                                            Message::YamlInputChanged(s, i, "replace".to_string())
                                        })
                                        .size(20)
                                    ]
                                    .spacing(10)
                                    .align_items(Alignment::Center)
                                ]
                                .spacing(8)]
                                .spacing(10)
                                .padding(20),
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

                let main_row = row![
                    nav_col,
                    match selected_nav.as_str() {
                        "eg-Settings" => settings_col,
                        // "eg-Config" => config_col,
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

fn read_to_triggers(file_path: PathBuf) -> EspansoYaml {
    let f = File::open(file_path).expect("Could not open file.");
    from_reader(f).expect("Could not read values.")
}

fn write_from_triggers(file_path: PathBuf, edited_file: EspansoYaml) {
    let f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .expect("Couldn't open file");
    to_writer(f, &edited_file).unwrap();
}

fn create_new_yml_file(file_path: PathBuf) {
    let f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .expect("Couldn't open file");
    to_writer(f, &EspansoYaml::default()).unwrap();
}

fn get_default_espanso_dir() -> String {
    // Get result of 'espanso path' command if possible
    let espanso_path_cmd = p_cmd::new("espanso")
        .arg("path")
        .output()
        .expect("failed to get path from espanso");
    let espanso_path_cmd_output =
        String::from_utf8(espanso_path_cmd.stdout).expect("Couldn't get espanso path");
    let espanso_path_array: Vec<&str> = espanso_path_cmd_output.split("\n").collect();
    if !espanso_path_array.is_empty() {
        if !espanso_path_array[0].is_empty() {
            if espanso_path_array[0].starts_with("Config:") {
                return espanso_path_array[0][8..].to_string();
            }
        }
    }

    // If that was unsuccessful, get the default path
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

// TODO: Could remove 'a here and make nav_to a String
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

mod style {
    use iced::widget::container;
    use iced::Theme;

    pub fn gray_background(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            ..Default::default()
        }
    }
}
