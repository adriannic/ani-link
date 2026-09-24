use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    Clone,
    Debug,
    EnumIter,
    EnumString,
    Display,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
)]
pub enum Locales {
    #[default]
    En,
    Es,
}

impl Locales {
    pub const fn next(self) -> Self {
        match self {
            Self::En => Self::Es,
            Self::Es => Self::En,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::En => Self::Es,
            Self::Es => Self::En,
        }
    }
}
