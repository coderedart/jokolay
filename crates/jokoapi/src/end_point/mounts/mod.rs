use joko_core::prelude::bitflags;

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
