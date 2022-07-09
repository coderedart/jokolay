

pub struct Plugin {

}

pub struct PluginInfo { // json file
    pub name: String,
    pub version: semver::Version,
    pub short_description: String,
    pub description: String,
    pub author: String,
    pub website: Option<url::Url>

}


// repository will act as the 

pub struct Repository {
    pub previous_check: std::time::Instant,
    pub latest: PackageList,
    pub mirrors: Vec<url::Url>
}


// serde on this
pub struct PackageList {
    pub package_entries: std::collections::BTreeMap<String, PackageEntry> // entry plugin name : entry
}

pub struct PackageEntry {
    pub version: semver::Version,
    pub download_link: url::Url,
    pub author: String,
    pub short_description: String,
    pub description: String,
    pub website: Option<url::Url>
    
}



