/// represents a plugin and its mlua state
pub struct Plugin {
    pub name: String,
}

bitflags::bitflags! {
    pub struct Permissions: u64 {
        const MUMBLE_LINK = 1 ;
        const GW2_API = 1 << 1;
        const FILE_SYSTEM = 1 << 2;

    }
}
