use iced::theme::Theme;
use iced::widget::{button, column, container, row, text, text_input, Container};
use iced::{alignment, window, Alignment, Application, Command, Element, Length, Settings};
// use once_cell::sync::Lazy;

// static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn main() -> iced::Result {
    EGUI::run(Settings {
        window: window::Settings {
            size: (1024, 768), // Width x Height
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
    folder_loc: String,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
}

impl Application for EGUI {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (EGUI::Loading, Command::none())
    }

    fn title(&self) -> String {
        String::from("espansoGUI")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            EGUI::Loading => Command::none(),
            EGUI::Loaded(state) => match message {
                Message::InputChanged(value) => {
                    state.folder_loc = value;
                    Command::none()
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            EGUI::Loading => container(
                text("Loading...")
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .size(50),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .into(),
            EGUI::Loaded(State { folder_loc, .. }) => {
                let nav_col = column![
                    text("Files").size(20),
                    column![text("JA.yaml").size(16)].padding([0, 0, 0, 10]),
                    text("Preferences").size(20),
                    text("Settings").size(20)
                ]
                .padding(20)
                .align_items(Alignment::Start);

                let settings_col = column![
                    row![text("Settings").size(25)].padding([0, 0, 20, 0]),
                    column![
                        text("espanso is not running").size(20),
                        text("Location").size(20),
                        text_input("What needs to be done?", folder_loc)
                            .on_input(Message::InputChanged)
                            //.on_submit(Message::CreateTask)
                            .padding(15)
                            .size(30),
                    ]
                    .spacing(10)
                    .padding([0, 0, 0, 20]),
                    // button("Decrement").on_press(Message::DecrementPressed)
                ]
                .padding([20, 20, 20, 40])
                .align_items(Alignment::Start);

                let main_row = row![nav_col, settings_col];

                Container::new(main_row)
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .into()
            }
        }
    }
}
