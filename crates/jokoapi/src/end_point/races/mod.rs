use std::str::FromStr;

use crate::prelude::*;

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Race {
    ASURA = 1 << 0,
    CHARR = 1 << 2,
    HUMAN = 1 << 3,
    NORN = 1 << 4,
    SYLVARI = 1 << 5,
}

impl FromStr for Race {
    type Err = &'static str;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "asura" => Self::ASURA,
            "charr" => Self::CHARR,
            "human" => Self::HUMAN,
            "norn" => Self::NORN,
            "sylvari" => Self::SYLVARI,
            _ => return Err("invalid race string"),
        })
    }
}

impl AsRef<str> for Race {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::ASURA => "asura",
            Self::CHARR => "charr",
            Self::HUMAN => "human",
            Self::NORN => "norn",
            Self::SYLVARI => "sylvari",
        }
    }
}
impl ToString for Race {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
