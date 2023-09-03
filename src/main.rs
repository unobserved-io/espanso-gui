mod triggers;

use dirs::config_dir;
use home;
use iced::theme::Theme;
use iced::widget::{
    button, column, container, row, text, text_input, Button, Column, Container, Scrollable, Space,
};
use iced::{
    alignment, window, Alignment, Application, Command, Element, Length, Renderer, Settings,
};
use rfd::FileDialog;
use serde_yaml::{self, Value};
use std::path::PathBuf;
use triggers::Triggers;
use walkdir::WalkDir;

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
    selected_file: PathBuf,
    original_file: Triggers,
    match_files: Vec<String>,
}

impl State {
    fn new() -> Self {
        if valid_espanso_dir(get_default_espanso_dir()) {
            State {
                espanso_loc: get_default_espanso_dir(),
                selected_nav: "eg-Settings".to_string(),
                selected_file: PathBuf::new(),
                original_file: Triggers::new(),
                match_files: {
                    let default_path = PathBuf::from(get_default_espanso_dir());
                    get_all_match_file_stems(default_path.join("match"))
                },
            }
        } else {
            State {
                espanso_loc: String::new(),
                selected_nav: "eg-Settings".to_string(),
                selected_file: PathBuf::new(),
                original_file: String::new(),
                match_files: Vec::new(),
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    AddPairPressed,
    InputChanged(String),
    YamlInputChanged(String),
    BrowsePressed,
    SettingsSavePressed,
    NavigateTo(String),
    UndoPressed,
    SaveFilePressed,
}

impl Application for EGUI {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (EGUI::Loaded(State::new()), Command::none())
    }

    fn title(&self) -> String {
        String::from("espansoGUI")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            EGUI::Loading => Command::none(),
            EGUI::Loaded(state) => match message {
                Message::AddPairPressed => Command::none(),
                Message::InputChanged(value) => {
                    state.espanso_loc = value;
                    Command::none()
                }
                Message::YamlInputChanged(value) => Command::none(),
                Message::NavigateTo(value) => {
                    state.selected_nav = value.clone();
                    let espanso_loc = state.espanso_loc.clone();
                    match value.as_str() {
                        "eg-Preferences" => {
                            state.selected_file = if espanso_loc.ends_with("/") {
                                PathBuf::from(espanso_loc + "config/default.yml")
                            } else {
                                PathBuf::from(espanso_loc + "/config/default.yml")
                            };
                        }
                        "eg-Settings" => state.selected_file = PathBuf::new(),
                        _ => {
                            state.selected_file = if espanso_loc.ends_with("/") {
                                PathBuf::from(espanso_loc + "match/" + &state.selected_nav + ".yml")
                            } else {
                                PathBuf::from(
                                    espanso_loc + "/match/" + &state.selected_nav + ".yml",
                                )
                            };
                            state.original_file = read_file_to_string(state.selected_file.clone());
                        }
                    }
                    Command::none()
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
                            state.espanso_loc = espanso_dir.into_os_string().into_string().unwrap();
                        } else {
                            // TODO: Show invalid directory
                        }
                    }

                    Command::none()
                }
                Message::SettingsSavePressed => {
                    if valid_espanso_dir(state.espanso_loc.clone()) {
                        state.match_files = get_all_match_file_stems(
                            PathBuf::from(state.espanso_loc.clone()).join("match"),
                        )
                    }

                    Command::none()
                }
                Message::UndoPressed => Command::none(),
                Message::SaveFilePressed => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let nav_col = column![
            text("Files").size(20),
            column![text("JA.yaml").size(16)].padding([0, 0, 0, 10]),
            text("Preferences").size(20),
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
                selected_file,
                original_file: file_string,
                match_files,
                ..
            }) => {
                let mut nav_col = column![text("Files").size(20),]
                    .spacing(12)
                    .padding(20)
                    .align_items(Alignment::Start);
                let mut yml_files_col: Column<'_, Message, Renderer> =
                    Column::new().spacing(8).padding([0, 0, 0, 10]);
                for yml_file in match_files {
                    yml_files_col = yml_files_col.push(nav_button(yml_file, yml_file));
                }
                nav_col = nav_col.push(yml_files_col);
                nav_col = nav_col.push(nav_button("Preferences", "eg-Preferences"));
                nav_col = nav_col.push(nav_button("Settings", "eg-Settings"));

                let settings_col = column![
                    row![text("Settings").size(25)].padding([0, 0, 20, 0]),
                    column![
                        text("espanso is not running").size(20),
                        row![
                            text("Location").size(20),
                            Space::new(10, 0),
                            text_input("", espanso_loc)
                                .on_input(Message::InputChanged)
                                .size(20),
                            Space::new(10, 0),
                            button("Browse").on_press(Message::BrowsePressed),
                        ]
                        .align_items(Alignment::Center),
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

                // let preferences_col = column![]
                //     .padding([20, 20, 20, 40])
                //     .width(Length::Fill)
                //     .align_items(Alignment::Start);

                let mut all_trigger_replace_rows: Column<'_, Message, Renderer> =
                    Column::new().spacing(8).padding([0, 0, 0, 10]);
                if !selected_nav.is_empty() && selected_nav != "eg-Settings" {
                    let yaml_data = yaml_from_string(file_string.clone());
                    let yaml_data = yaml_data.as_mapping();
                    if yaml_data.is_some() {
                        let yaml_data = yaml_data.unwrap();
                        if let Some(trigger_replace_list) = yaml_data["matches"].as_sequence() {
                            all_trigger_replace_rows = all_trigger_replace_rows.push(row![
                                button("+ Add").on_press(Message::AddPairPressed),
                                text(format!("Items: {}", trigger_replace_list.len())),
                                Space::new(Length::Fill, 0),
                                button("Undo").on_press(Message::UndoPressed),
                                button("Save").on_press(Message::SaveFilePressed),
                            ]);
                            for trigger_replace in trigger_replace_list {
                                let trigger_str = trigger_replace["trigger"].as_str();
                                let replace_str = trigger_replace["replace"].as_str();
                                if trigger_str.is_some() && replace_str.is_some() {
                                    let trigger_str = trigger_str.unwrap();
                                    let replace_str = replace_str.unwrap();
                                    let trigger_replace_container: Container<
                                        '_,
                                        Message,
                                        Renderer,
                                    > = Container::new(
                                        row![column![
                                            row![
                                                text("Trigger:").size(20).width(90),
                                                text_input(trigger_str, trigger_str)
                                                    .on_input(Message::YamlInputChanged)
                                                    .size(20)
                                            ],
                                            row![
                                                text("Replace:").size(20).width(75),
                                                text_input(replace_str, replace_str)
                                                    .on_input(Message::YamlInputChanged)
                                                    .size(20)
                                            ]
                                            .spacing(10)
                                            .align_items(Alignment::Center)
                                        ]
                                        .spacing(8)]
                                        .spacing(10)
                                        .padding(20),
                                    )
                                    .style(style::gray_background);
                                    all_trigger_replace_rows =
                                        all_trigger_replace_rows.push(trigger_replace_container);
                                }
                            }
                        }
                    }
                }

                let open_file_col = column![Scrollable::new(
                    all_trigger_replace_rows.padding([20, 20, 20, 40])
                )]
                .width(Length::Fill)
                .align_items(Alignment::Start);

                let main_row = row![
                    nav_col,
                    match selected_nav.as_str() {
                        "eg-Settings" => settings_col,
                        // "eg-Preferences" => preferences_col,
                        _ => open_file_col,
                    }
                ];

                Container::new(main_row)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .into()
            }
        }
    }
}

// Could remove 'a here and make nav_to a String
fn nav_button<'a>(text: &'a str, nav_to: &'a str) -> Button<'a, Message> {
    button(text).on_press(Message::NavigateTo(nav_to.to_string()))
}

fn read_to_triggers(file_path: PathBuf) {
    let f = std::fs::File::open(file_path).expect("Could not open file.");
    let scrape_triggers: Triggers = serde_yaml::from_reader(f).expect("Could not read values.");
    println!("{:?}", scrape_triggers);
}

fn read_file_to_string(file_path: PathBuf) -> String {
    read_to_triggers(file_path.clone());
    std::fs::read_to_string(file_path).expect("Failed to read file")
}

fn yaml_from_string(file_contents: String) -> Value {
    serde_yaml::from_str(&file_contents).expect("Failed to parse YAML")
}

fn get_default_espanso_dir() -> String {
    let mut default_loc = String::new();
    if let Some(config_dir) = config_dir() {
        let default_path = config_dir.join("espanso");
        default_loc = default_path.display().to_string();
    }

    default_loc
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
