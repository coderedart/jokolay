use bevy::utils::HashSet;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;
pub mod pack;

/*
marker pack repos: installation, updates, deletion.
marker pack editing, saving, reloading.
marker pack activation data and cleanup.
marker pack rendering based on character + their logged in map + camera render target (window surface)
*/

/*
Activation data:
per account
per character
per character per map instance

*/

// /// The primary struct which manages Marker Packs.
// /// in case of a Marker pack available in more than one repository, the first one would be selected
// ///
// pub struct MarkerManager {
//     /// packs that are live
//     pub packs: BTreeMap<String, Pack>,
//     /// repositories from where we can get marker pack lists
//     pub repos: Vec<Repository>,
//     /// marker pack installed version kept separate
//     pub installed: BTreeMap<String, Version>
// }

/// it represents a list of marker packs.
/// the mirrors are host URLs which point to a json file consisting of a PackList
/// the first mirror which works and provides us with a PackList will be used.
/// The functionality is derived from the concepts of software repos in Arch linux which
/// has core, main, extra, community as official and we can addd additional repositories
/// like Endeavour or chaotic etc..
///
/// This will allow others to host their own repositories for custom marker pack lists
/// and the repo maintainers can sort of "vouch" for the quality or security of the pack.
///
pub struct Repository {
    /// Name of the repository
    pub name: String,
    /// must point to a json. which will be in the format of a `PackList`
    pub url: Url,
}

/// This is the list of Markerpacks in a repository.
#[derive(Serialize, Deserialize)]
pub struct PackList {
    /// if the jmf_supported_version is greater than the version of the compiled
    /// jmf crate, it means we cannot use this list
    pub jmf_supported_version: Version,
    /// String: The name of the pack
    /// we only support a single latest version of marker pack entry.
    pub entries: BTreeMap<String, PackEntry>,
}

/// This represents a single Pack download info.
#[derive(Serialize, Deserialize)]
pub struct PackEntry {
    /// The version of the pack, that the download info refers to.
    pub version: Version,
    /// Each entry belongs to a particular version
    pub download_info: DownloadInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadInfo {
    RawUrlXML { url: Url },
    RawUrlJson { url: Url },
}
pub struct AccountActivation {
    pub date: usize, // once day changes, empty the daily_reset set
    pub daily_reset: HashSet<UniqueMarkerID>,
    pub permanent: HashSet<UniqueMarkerID>,
    pub timer: BTreeMap<UniqueMarkerID, usize>, // unix milliseconds timestamp when marker should be reactivated (removed from this map)
    pub instance: BTreeMap<u32, HashSet<UniqueMarkerID>>, // key is instance ip. values are a set of markers that activated in that instance
}

pub struct CharacterActivation {
    pub date: usize, // once day changes, empty the daily_reset set
    pub daily_reset: HashSet<UniqueMarkerID>,
}

#[derive(Debug, Hash, PartialEq, PartialOrd)]
pub struct UniqueMarkerID {
    pub index: usize,
    pub map: u16,
}
pub struct ActivationData {
    /// key is Account name. Value is Activation data.
    pub account_data: std::collections::BTreeMap<String, AccountActivation>,
    /// Key is Character name. Value is activation data.
    pub character_data: std::collections::BTreeMap<String, CharacterActivation>,
}
