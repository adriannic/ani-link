use std::{fmt, str::FromStr, sync::LazyLock};

use clap::ValueEnum;
use iced::{Color, Theme, color, theme::Palette};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub static ANILINK_THEME: LazyLock<Theme> = LazyLock::new(|| {
    iced::Theme::custom(
        "custom".into(),
        Palette {
            background: Color::from_rgba(
                f32::from(0x03_u16) / 255.0,
                f32::from(0x04_u16) / 255.0,
                f32::from(0x0D_u16) / 255.0,
                0.75,
            ), // #03040D
            text: color!(0xD9_CBD2),    // #D9CBD2
            primary: color!(0xA4_9029), // #A49029
            success: color!(0x00_FF00), // #00FF00
            danger: color!(0xFF_0000),  // #FF0000
        },
    )
});

#[derive(
    ValueEnum, Clone, Debug, EnumIter, Copy, Serialize, Deserialize, PartialEq, Eq, Default,
)]
#[clap(rename_all = "PascalCase")]
pub enum Themes {
    #[default]
    AniLink,
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

#[allow(clippy::from_over_into)]
impl Into<iced::Theme> for Themes {
    fn into(self) -> iced::Theme {
        match self {
            Self::AniLink => ANILINK_THEME.clone(),
            Self::Light => Theme::Light,
            Self::Dark => Theme::Dark,
            Self::Dracula => Theme::Dracula,
            Self::Nord => Theme::Nord,
            Self::SolarizedLight => Theme::SolarizedLight,
            Self::SolarizedDark => Theme::SolarizedDark,
            Self::GruvboxLight => Theme::GruvboxLight,
            Self::GruvboxDark => Theme::GruvboxDark,
            Self::CatppuccinLatte => Theme::CatppuccinLatte,
            Self::CatppuccinFrappe => Theme::CatppuccinFrappe,
            Self::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            Self::CatppuccinMocha => Theme::CatppuccinMocha,
            Self::TokyoNight => Theme::TokyoNight,
            Self::TokyoNightStorm => Theme::TokyoNightStorm,
            Self::TokyoNightLight => Theme::TokyoNightLight,
            Self::KanagawaWave => Theme::KanagawaWave,
            Self::KanagawaDragon => Theme::KanagawaDragon,
            Self::KanagawaLotus => Theme::KanagawaLotus,
            Self::Moonfly => Theme::Moonfly,
            Self::Nightfly => Theme::Nightfly,
            Self::Oxocarbon => Theme::Oxocarbon,
            Self::Ferra => Theme::Ferra,
        }
    }
}

impl fmt::Display for Themes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AniLink => "AniLink",
                Self::Light => "Light",
                Self::Dark => "Dark",
                Self::Dracula => "Dracula",
                Self::Nord => "Nord",
                Self::SolarizedLight => "SolarizedLight",
                Self::SolarizedDark => "SolarizedDark",
                Self::GruvboxLight => "GruvboxLight",
                Self::GruvboxDark => "GruvboxDark",
                Self::CatppuccinLatte => "CatppuccinLatte",
                Self::CatppuccinFrappe => "CatppuccinFrappe",
                Self::CatppuccinMacchiato => "CatppuccinMacchiato",
                Self::CatppuccinMocha => "CatppuccinMocha",
                Self::TokyoNight => "TokyoNight",
                Self::TokyoNightStorm => "TokyoNightStorm",
                Self::TokyoNightLight => "TokyoNightLight",
                Self::KanagawaWave => "KanagawaWave",
                Self::KanagawaDragon => "KanagawaDragon",
                Self::KanagawaLotus => "KanagawaLotus",
                Self::Moonfly => "Moonfly",
                Self::Nightfly => "Nightfly",
                Self::Oxocarbon => "Oxocarbon",
                Self::Ferra => "Ferra",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseThemeError;

impl FromStr for Themes {
    type Err = ParseThemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AniLink" => Ok(Self::AniLink),
            "Light" => Ok(Self::Light),
            "Dark" => Ok(Self::Dark),
            "Dracula" => Ok(Self::Dracula),
            "Nord" => Ok(Self::Nord),
            "SolarizedLight" => Ok(Self::SolarizedLight),
            "SolarizedDark" => Ok(Self::SolarizedDark),
            "GruvboxLight" => Ok(Self::GruvboxLight),
            "GruvboxDark" => Ok(Self::GruvboxDark),
            "CatppuccinLatte" => Ok(Self::CatppuccinLatte),
            "CatppuccinFrappe" => Ok(Self::CatppuccinFrappe),
            "CatppuccinMacchiato" => Ok(Self::CatppuccinMacchiato),
            "CatppuccinMocha" => Ok(Self::CatppuccinMocha),
            "TokyoNight" => Ok(Self::TokyoNight),
            "TokyoNightStorm" => Ok(Self::TokyoNightStorm),
            "TokyoNightLight" => Ok(Self::TokyoNightLight),
            "KanagawaWave" => Ok(Self::KanagawaWave),
            "KanagawaDragon" => Ok(Self::KanagawaDragon),
            "KanagawaLotus" => Ok(Self::KanagawaLotus),
            "Moonfly" => Ok(Self::Moonfly),
            "Nightfly" => Ok(Self::Nightfly),
            "Oxocarbon" => Ok(Self::Oxocarbon),
            "Ferra" => Ok(Self::Ferra),
            _ => Err(ParseThemeError),
        }
    }
}

impl Themes {
    pub const fn next(self) -> Self {
        match self {
            Self::AniLink => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::Dracula,
            Self::Dracula => Self::Nord,
            Self::Nord => Self::SolarizedLight,
            Self::SolarizedLight => Self::SolarizedDark,
            Self::SolarizedDark => Self::GruvboxLight,
            Self::GruvboxLight => Self::GruvboxDark,
            Self::GruvboxDark => Self::CatppuccinLatte,
            Self::CatppuccinLatte => Self::CatppuccinFrappe,
            Self::CatppuccinFrappe => Self::CatppuccinMacchiato,
            Self::CatppuccinMacchiato => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::TokyoNight,
            Self::TokyoNight => Self::TokyoNightStorm,
            Self::TokyoNightStorm => Self::TokyoNightLight,
            Self::TokyoNightLight => Self::KanagawaWave,
            Self::KanagawaWave => Self::KanagawaDragon,
            Self::KanagawaDragon => Self::KanagawaLotus,
            Self::KanagawaLotus => Self::Moonfly,
            Self::Moonfly => Self::Nightfly,
            Self::Nightfly => Self::Oxocarbon,
            Self::Oxocarbon => Self::Ferra,
            Self::Ferra => Self::AniLink,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::AniLink => Self::Ferra,
            Self::Light => Self::AniLink,
            Self::Dark => Self::Light,
            Self::Dracula => Self::Dark,
            Self::Nord => Self::Dracula,
            Self::SolarizedLight => Self::Nord,
            Self::SolarizedDark => Self::SolarizedLight,
            Self::GruvboxLight => Self::SolarizedDark,
            Self::GruvboxDark => Self::GruvboxLight,
            Self::CatppuccinLatte => Self::GruvboxDark,
            Self::CatppuccinFrappe => Self::CatppuccinLatte,
            Self::CatppuccinMacchiato => Self::CatppuccinFrappe,
            Self::CatppuccinMocha => Self::CatppuccinMacchiato,
            Self::TokyoNight => Self::CatppuccinMocha,
            Self::TokyoNightStorm => Self::TokyoNight,
            Self::TokyoNightLight => Self::TokyoNightStorm,
            Self::KanagawaWave => Self::TokyoNightLight,
            Self::KanagawaDragon => Self::KanagawaWave,
            Self::KanagawaLotus => Self::KanagawaDragon,
            Self::Moonfly => Self::KanagawaLotus,
            Self::Nightfly => Self::Moonfly,
            Self::Oxocarbon => Self::Nightfly,
            Self::Ferra => Self::Oxocarbon,
        }
    }
}
