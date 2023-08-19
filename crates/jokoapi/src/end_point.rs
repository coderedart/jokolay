// use async_trait::async_trait;
// use miette::eyre::WrapErr;

// use itertools::Itertools;
use serde::de::DeserializeOwned;
// use surf::Client;
use std::fmt::Display;

pub mod colors;
pub mod daily_crafting;
pub mod items;
pub mod minis;
pub mod outfits;
pub mod quaggans;
pub mod races;
pub mod worlds;

// #[async_trait]
pub trait EndPointAuthId {
    type Id;
    type RType;
    fn get_url(id: &Self::Id) -> String;
    // async fn get_auth_with_id(
    //     client: Client,
    //     api_key: &str,
    //     id: &Self::Id,
    // ) -> Result<Self::RType>
    // where
    //     Self::RType: DeserializeOwned,
    //     Self::Id: Display + Send + Sync,
    // {
    //     let res = client
    //         .get(&Self::get_url(id))
    //         .aut(api_key)
    //         .send()
    //         .await?;
    //     Ok(res.json().await.wrap_err(format!(
    //         "couldn't convert json result to rust type {}",
    //         type_name::<Self::RType>()
    //     ))?)
    // }
}

// #[async_trait]
pub trait EndPointAuthIds {
    type Id: Display + Send + Sync;
    type RType;
    fn get_url() -> &'static str;
    // async fn get_auth_with_id(
    //     client: Client,
    //     api_key: &str,
    //     ids: &[Self::Id],
    // ) -> Result<Self::RType>
    // where
    //     Self::RType: DeserializeOwned,
    //     Self::Id: Display + Send + Sync,
    // {
    //     let res = client
    //         .get(Self::get_url())
    //         .bearer_auth(api_key)
    //         .query(&[("ids", ids.iter().join(","))])
    //         .send()
    //         .await?;
    //     Ok(res.json().await.wrap_err(format!(
    //         "couldn't convert json result to rust type {}",
    //         type_name::<Self::RType>()
    //     ))?)
    // }
}
// #[async_trait]
pub trait EndPointAuth {
    type RType: DeserializeOwned;
    fn get_url() -> &'static str;
    // async fn get_auth(client: Client, api_key: &str) -> Result<Self::RType>
    // where
    //     Self::RType: DeserializeOwned,
    // {
    //     let res = client
    //         .get(Self::get_url())
    //         .bearer_auth(api_key)
    //         .send()
    //         .await?;
    //     Ok(res.json().await.wrap_err(format!(
    //         "couldn't convert json result to rust type {}",
    //         type_name::<Self::RType>()
    //     ))?)
    // }
}
// #[async_trait]
pub trait EndPointIds {
    type Id: Display + Send + Sync;
    type RType: DeserializeOwned;
    fn get_url() -> &'static str;
    // async fn get_with_id(client: Client, ids: &[Self::Id]) -> Result<Self::RType>
    // where
    //     Self::RType: DeserializeOwned,
    //     Self::Id: Display + Send + Sync,
    // {
    //     let res = client
    //         .get(Self::get_url())
    //         .query(&[("ids", ids.iter().join(","))])
    //         .send()
    //         .await?;
    //     Ok(res.json().await.wrap_err(format!(
    //         "couldn't convert json result to rust type {}",
    //         type_name::<Self::RType>()
    //     ))?)
    // }
}
// #[async_trait]
pub trait EndPoint {
    type RType: DeserializeOwned;
    fn get_url() -> &'static str;
    // async fn get(client: &Client) -> Result<Self::RType>
    // where
    //     Self::RType: DeserializeOwned,
    // {
    //     let res = client.get(Self::get_url()).send().await?;
    //     Ok(res.json().await.wrap_err(format!(
    //         "couldn't convert json result to rust type {}",
    //         type_name::<Self::RType>()
    //     ))?)
    // }
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
