use iced::Task;

use crate::app::{self, Message};

#[derive(Default)]
pub enum AppUpdate {
    #[default]
    None,
    Page(Box<dyn Page>),
    Task(Task<Message>),
    Both((Box<dyn Page>, Task<Message>))
}

pub trait Page {
    fn view(&self) -> iced::Element<'_, app::Message>;
    fn update(&mut self, message: app::Message) -> AppUpdate;
    fn subscription(&self) -> iced::Subscription<app::Message>;
    fn theme(&self) -> iced::Theme;
}
