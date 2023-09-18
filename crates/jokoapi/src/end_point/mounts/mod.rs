use std::str::FromStr;

use crate::prelude::*;

#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Mount {
    Raptor = 1 << 0,
    Springer = 1 << 1,
    Skimmer = 1 << 2,
    Jackal = 1 << 3,
    Griffon = 1 << 4,
    RollerBeetle = 1 << 5,
    Warclaw = 1 << 6,
    Skyscale = 1 << 7,
    Skiff = 1 << 8,
    SiegeTurtle = 1 << 9,
}

impl FromStr for Mount {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "raptor" => Self::Raptor,
            "springer" => Self::Springer,
            "skimmer" => Self::Skimmer,
            "jackal" => Self::Jackal,
            "griffon" => Self::Griffon,
            "rollerbeetle" => Self::RollerBeetle,
            "warclaw" => Self::Warclaw,
            "skyscale" => Self::Skyscale,
            "skiff" => Self::Skiff,
            "siegeturtle" => Self::SiegeTurtle,
            _ => return Err("invalid mount string"),
        })
    }
}
impl AsRef<str> for Mount {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Raptor => "raptor",
            Self::Springer => "springer",
            Self::Skimmer => "skimmer",
            Self::Jackal => "jackal",
            Self::Griffon => "griffon",
            Self::RollerBeetle => "rollerbeetle",
            Self::Warclaw => "warclaw",
            Self::Skyscale => "skyscale",
            Self::Skiff => "skiff",
            Self::SiegeTurtle => "siegeturtle",
        }
    }
}
impl ToString for Mount {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
