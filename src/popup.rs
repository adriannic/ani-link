use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub struct Popup {
    title: String,
    text: String,
}

impl Popup {
    pub fn new(title: String, text: String) -> Self {
        Self { title, text }
    }
}

impl Widget for Popup {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let popup = Paragraph::new(self.text)
            .block(
                Block::bordered()
                    .title(Line::from(self.title).centered().white())
                    .border_set(border::THICK)
                    .border_style(Style::new().green())
                    .on_black(),
            )
            .bold()
            .centered();

        popup.render(area, buf);
    }
}

pub fn get_popup_area(area: Rect, x: u16, y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
