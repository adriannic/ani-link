use colors_transform::{Color, Rgb};
use iced::{
    Border, Element, Length,
    overlay::menu,
    widget::{Button, Container, Space, button, column, container, pick_list, row, text},
};
use strum::IntoEnumIterator;

pub fn highlight(color: iced::Color, percentage: f32) -> iced::Color {
    let [r, g, b, a] = color.into_rgba8();
    let text_color = Rgb::from(r as f32, g as f32, b as f32);
    let is_text_dark = (text_color.to_hsl().get_lightness() / 100.0).round() as u8 == 0;

    let (r, g, b) = text_color
        .to_hsl()
        .lighten(if is_text_dark {
            -percentage
        } else {
            percentage
        })
        .to_rgb()
        .as_tuple();

    iced::Color::from_rgba(r / 255.0, g / 255.0, b / 255.0, a as f32 / 255.0)
}

pub fn square_box<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    container(
        Container::new(content)
            .style(move |theme| container::Style {
                border: Border {
                    color: theme.palette().primary,
                    width: 3.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(3)
}

pub fn help_text<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    Container::new(content).style(|theme: &iced::Theme| container::Style {
        text_color: Some(theme.palette().primary),
        ..Default::default()
    })
}

pub fn transparent_button<'a, Message: 'a>(content: &str, selected: bool) -> Button<'a, Message> {
    let string = if selected {
        format!("> {content}")
    } else {
        format!("  {content}")
    };

    button(text(string))
        .style(move |theme: &iced::Theme, status| button::Style {
            text_color: if selected || matches!(status, button::Status::Hovered) {
                highlight(theme.palette().text, 20.0)
            } else {
                theme.palette().text
            },
            ..Default::default()
        })
        .padding(0)
}

pub fn transparent_button_cond<'a, Message: 'a>(
    content: &str,
    selected: impl Fn() -> bool,
) -> Button<'a, Message> {
    let selected = selected();
    let string = if selected {
        format!("> {content}")
    } else {
        format!("  {content}")
    };

    button(text(string))
        .style(move |theme: &iced::Theme, status| button::Style {
            text_color: if selected || matches!(status, button::Status::Hovered) {
                highlight(theme.palette().text, 20.0)
            } else {
                theme.palette().text
            },
            ..Default::default()
        })
        .padding(0)
}

pub fn options_item<'a, T: IntoEnumIterator + ToString, Message: Clone + 'a>(
    name: &str,
    selected: bool,
    current: Option<String>,
    callback: impl Fn(String) -> Message + 'a,
) -> Container<'a, Message> {
    Container::new(
        column![
            transparent_button(name, selected),
            row![
                Space::with_width(Length::Fixed(18.0)),
                pick_list(
                    T::iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                    current,
                    callback,
                )
                .menu_style(|theme: &iced::Theme| {
                    menu::Style {
                        background: iced::Background::Color(theme.palette().background)
                            .scale_alpha(1.0 / theme.palette().background.a),
                        text_color: theme.palette().text,
                        border: Border {
                            color: theme.palette().primary,
                            width: 1.0,
                            ..Default::default()
                        },
                        selected_background: iced::Background::Color(theme.palette().primary),
                        selected_text_color: theme
                            .palette()
                            .background
                            .scale_alpha(1.0 / theme.palette().background.a),
                    }
                })
                .style(|theme: &iced::Theme, _| {
                    pick_list::Style {
                        background: iced::Background::Color(theme.palette().background),
                        text_color: theme.palette().text,
                        placeholder_color: theme.palette().danger,
                        handle_color: theme.palette().primary,
                        border: Border {
                            color: theme.palette().primary,
                            width: 1.0,
                            ..Default::default()
                        },
                    }
                })
            ]
        ]
        .spacing(6),
    )
}
