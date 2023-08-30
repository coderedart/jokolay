// use async_trait::async_trait;
// use miette::eyre::WrapErr;

// use itertools::Itertools;
use crate::prelude::*;

pub use serde::{Deserialize, Serialize};

// pub mod colors;
// pub mod daily_crafting;
// pub mod items;
// pub mod minis;
// pub mod outfits;
// pub mod quaggans;
// pub mod races;
pub mod mounts;
pub mod races;
pub mod worlds;
const AUTHORIZATION_HEADER_NAME: &str = "Authorization";

/// We implement this for types which represent the data provided by a particular endpoint.
/// eg: We create a Color struct with all the fields we expect in color. Then, we simply impl this trait for that
/// This is useful for MOST (but not all) of the endpoints because of three properties
/// 1. When you go to endpoint url, we get a list of ids
/// 2. we can get an item of that type if we go to endpoint/id url, you always get the item type as return value
/// 3. when you add the `ids` query parameter like endpoint?ids=1,2,3, you will get a list of values which are of the item type
/// Obviously, to get that item type from json, it needs to impl Deserialize
///
/// If the endpoint doesn't need authentication, just use an empty string for the api_key
pub trait EndPoint: DeserializeOwned {
    /// The type of the id. For most items it is either a number of a string (or an enum when it is a fixed number of known static strings)
    type Id: Display + Send + Sync + DeserializeOwned;
    const URL: &'static str;
    const AUTH: bool;

    /// This function simply takes the [Self::URL], makes a request and returns a list of ids.
    fn get(client: &HttpClient, api_key: &str) -> Result<Vec<Self::Id>> {
        let req = client.get(Self::URL);
        let req = if Self::AUTH {
            req.set(AUTHORIZATION_HEADER_NAME, &format!("Bearer {api_key}"))
        } else {
            req
        };
        req.call().into_diagnostic()?.into_json().into_diagnostic()
    }
    fn get_id(client: &HttpClient, api_key: &str, id: &Self::Id) -> Result<Self> {
        let req = client.get(&format!("{}/{}", Self::URL, id));
        let req = if Self::AUTH {
            req.set(AUTHORIZATION_HEADER_NAME, &format!("Bearer {api_key}"))
        } else {
            req
        };
        req.call().into_diagnostic()?.into_json().into_diagnostic()
    }
    fn get_ids(client: HttpClient, api_key: &str, ids: &[Self::Id]) -> Result<Vec<Self>> {
        let mut ids_str = String::new();
        for id in ids {
            if !ids_str.is_empty() {
                ids_str.push(',');
            }
            ids_str.push_str(&format!("{id}"));
        }
        let req = client.get(Self::URL).query("ids", &ids_str);
        let req = if Self::AUTH {
            req.set(AUTHORIZATION_HEADER_NAME, &format!("Bearer {api_key}"))
        } else {
            req
        };
        req.call().into_diagnostic()?.into_json().into_diagnostic()
    }
}

/*

pub(crate) enum V2 {
    Account,
    AccountBank,
    AccountBuildstorage,
    AccountDailycrafting,
    AccountDungeons,
    AccountDyes,
    AccountEmotes,
    AccountFinishers,
    AccountGliders,
    AccountHome,
    AccountHomeCats,
    AccountHomeNodes,
    AccountInventory,
    AccountLuck,
    AccountMailcarriers,
    AccountMapchests,
    AccountMasteries,
    AccountMasteryPoints,
    AccountMaterials,
    AccountMinis,
    AccountMounts,
    AccountMountsSkins,
    AccountMountsTypes,
    AccountNovelties,
    AccountOutfits,
    AccountPvpHeroes,
    AccountRaids,
    AccountRecipes,
    AccountSkins,
    AccountTitles,
    AccountWallet,
    AccountWorldbosses,
    Achievements,
    AchievementsCategories,
    AchievementsDaily,
    AchievementsDailyTomorrow,
    AchievementsGroups,
    Backstory,
    BackstoryAnswers,
    BackstoryQuestions,
    Build,
    Characters,
    CharactersBackstory,
    CharactersBuildtabs,
    CharactersCore,
    CharactersCrafting,
    CharactersEquipment,
    CharactersEquipmenttabs,
    CharactersHeropoints,
    CharactersInventory,
    CharactersQuests,
    CharactersRecipes,
    CharactersSab,
    CharactersSkills,
    CharactersSpecializations,
    CharactersTraining,
    Colors,
    Commerce,
    CommerceDelivery,
    CommerceExchange,
    CommerceExchangeCoins,
    CommerceExchangeGems,
    CommerceListings,
    CommercePrices,
    CommerceTransactions,
    Continents,
    Createsubtoken,
    Currencies,
    Dailycrafting,
    Dungeons,
    Emblem,
    EmblemBackgrounds,
    EmblemForegrounds,
    Emotes,
    Files,
    Finishers,
    Gliders,
    Guild,
    GuildLog,
    GuildMembers,
    GuildPermissions,
    GuildRanks,
    GuildStash,
    GuildStorage,
    GuildTeams,
    GuildTreasury,
    GuildUpgrades,
    GuildSearch,
    HomeCats,
    HomeNodes,
    Items,
    Itemstats,
    Legends,
    Mailcarriers,
    Mapchests,
    Maps,
    Masteries,
    Materials,
    Minis,
    Mounts,
    MountsSkins,
    MountsTypes,
    Novelties,
    Outfits,
    Pets,
    Professions,
    Pvp,
    PvpAmulets,
    PvpGames,
    PvpHeroes,
    PvpRanks,
    PvpSeasons,
    PvpSeasonsLeaderboards,
    PvpStandings,
    PvpStats,
    Quaggans(Quaggans),
    Quests,
    Races,
    Raids,
    Recipes,
    RecipesSearch,
    Skills,
    Skins,
    Specializations,
    Stories,
    StoriesSeasons,
    Titles,
    TokenInfo,
    Traits,
    Worldbosses,
    Worlds,
    Wvw,
    WvwAbilities,
    WvwMatches,
    WvwMatchesStatsTeams,
    WvwObjectives,
    WvwRanks,
    WvwUpgrades,
}
*/
