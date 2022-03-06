use std::collections::{BTreeSet, HashMap, HashSet};

pub struct Player {

}

pub struct Profile {
    pub id: String,
    pub api_key: String,
    pub characters: BTreeSet<String>
}


/*
game api data. achievements, maps, items etc..
account api data. achievements, mastery, chars, inventories, trading post listings etc..
account markerpack data -> enabled categories, activation data.
account details -> name, chars list, apikey
 */