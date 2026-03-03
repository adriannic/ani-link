use crate::app;

pub trait Page {
    fn view(&self) -> iced::Element<'_, app::Message>;
    fn update(&mut self, message: app::Message) -> Option<Box<dyn Page>>;
    fn subscription(&self) -> iced::Subscription<app::Message>;
    fn theme(&self) -> iced::Theme;
}
