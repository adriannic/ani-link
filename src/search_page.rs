use iced::{
    Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    widget::{Column, Space, column, container, horizontal_rule, row, text, text_input},
};
use reqwest::blocking::Client;

use crate::{
    app,
    config::Config,
    main_menu_page,
    page::Page,
    presets::{help_text, square_box, transparent_button},
    scraper::anime::Anime,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SearchState {
    Searching,
    Choosing,
}

pub struct SearchPage {
    pub config: Config,
    pub client: Client,
    pub theme: Theme,
    pub anime_list: Vec<Anime>,
    pub search_state: SearchState,
    pub query: String,
    pub anime_state: u64,
    pub filtered_list: Vec<Anime>,
}

impl Page for SearchPage {
    fn view(&self) -> iced::Element<'_, crate::app::Message> {
        row![
            square_box(
                column![
                    transparent_button("Buscar", true)
                        .width(Length::Fill)
                        .on_press(app::Message::MainMenu(main_menu_page::Message::Select(
                            main_menu_page::Selection::Search
                        ))),
                    transparent_button("Opciones", false)
                        .width(Length::Fill)
                        .on_press(app::Message::MainMenu(main_menu_page::Message::Select(
                            main_menu_page::Selection::Options
                        ))),
                    transparent_button("Salir", false)
                        .width(Length::Fill)
                        .on_press(app::Message::MainMenu(main_menu_page::Message::Select(
                            main_menu_page::Selection::Exit
                        ))),
                    Space::with_height(Length::Fill),
                    container(
                        column![
                            container("Subir:"),
                            help_text("↑ K"),
                            "",
                            container("Bajar:"),
                            help_text("↓ J"),
                            "",
                            container("Confirmar:"),
                            help_text("→ L Enter"),
                            "",
                            container("Salir:"),
                            help_text("← H Esc"),
                            "",
                        ]
                        .align_x(Horizontal::Center)
                        .width(Length::Fill)
                    )
                    .align_y(Vertical::Bottom)
                    .width(Length::Fill)
                ]
                .padding(6),
            )
            .width(Length::Fixed(120.0))
            .height(Length::Fill),
            square_box(column![
                text_input("Buscar", &self.query),
                horizontal_rule(1),
                Column::with_children(
                    self.filtered_list
                        .iter()
                        .map(|anime| Element::from(text(anime.names[0].clone())))
                        .collect::<Vec<_>>()
                )
            ])
        ]
        .padding(3)
        .into()
    }

    fn update(&mut self, message: crate::app::Message) -> Option<Box<dyn Page>> {
        todo!()
    }

    fn subscription(&self) -> iced::Subscription<crate::app::Message> {
        todo!()
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}
