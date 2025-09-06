use iced::{
    Border, Color, Element, Length,
    theme::Palette,
    widget::{Button, Container, button, container, text},
};

pub fn square_box<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    palette: Palette,
) -> Container<'a, Message> {
    container(
        Container::new(content)
            .style(move |_| container::Style {
                border: Border {
                    color: palette.primary,
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

pub fn transparent_button<'a, Message: 'a>(
    content: &str,
    palette: Palette,
    selected: bool,
) -> Button<'a, Message> {
    let string = if selected {
        format!("> {content}")
    } else {
        format!("  {content}")
    };

    button(text(string)).style(move |_, _| button::Style {
        text_color: if selected { Color::WHITE } else { palette.text },
        ..Default::default()
    }).padding(0)
}
