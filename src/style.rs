use iced::widget::container;
use iced::Theme;

pub fn gray_background(theme: &Theme) -> container::Appearance {
    let palette = theme.extended_palette();

    container::Appearance {
        background: Some(palette.background.weak.color.into()),
        ..Default::default()
    }
}
