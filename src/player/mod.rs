use std::collections::BTreeSet;

pub struct PlayerContextImpl {}

pub struct Profile {
    pub id: String,
    pub api_key: String,
    pub characters: BTreeSet<String>,
}

/*
game api data. achievements, maps, items etc..
account api data. achievements, mastery, chars, inventories, trading post listings etc..
account markerpack data -> enabled categories, activation data.
account details -> name, chars list, apikey
 */
pub struct MumbleContext {}
pub struct Gw2InstanceData {
    pub pid: u32,
    pub wid: u32,
    pub position: [f32; 2],
    pub size: [u32; 2],
    pub monitor: u8,
}
pub struct AccountContext {
    // api data
// marker activation data
// characters data
}
pub struct CharacterContext {
    // api data
// marker activation data
// character data
}

/*
there can be multiple gw2 instances
some of they could be using the same mumble link name
each gw2 instance should have a unique pid and wid
gw2 windows could be on different monitors
gw2 window could be minimized, not in focus, window mode, windowed fullscreen
any of the gw2 instances could crash/quit at any moment

 */
