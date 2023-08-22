use joko_core::prelude::bitflags;

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
