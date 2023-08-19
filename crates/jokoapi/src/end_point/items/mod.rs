use core::str;
use url::Url;

use super::{EndPoint, EndPointIds};

pub type ItemId = u32;
const E_P_URL: &str = const_format::concatcp!(crate::jokoapi::API_BASE_V2_URL, "/items");
#[derive(Serialize, Deserialize)]
pub struct Item {
    id: ItemId,
    chat_link: String,
    name: String,
    icon: Option<Url>,
    description: Option<String>,
    #[serde(rename = "type")]
    t: String,
    rarity: String,
    level: u32,
    vendor_value: u32,
    default_skin: Option<u32>,
    flags: Vec<String>,
    game_types: Vec<String>,
    restrictions: Vec<String>,
    upgrades_into: Option<Vec<ItemUpgrade>>,
    upgrades_from: Option<Vec<ItemUpgrade>>,
    details: serde_json::Value,
}
#[derive(Serialize, Deserialize)]
pub struct ItemUpgrade {
    upgrade: String,
    item_id: ItemId,
}

pub struct Items;

impl EndPoint for Items {
    type RType = Vec<ItemId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Items {
    type Id = ItemId;
    type RType = Vec<Item>;

    fn get_url() -> &'static str {
        E_P_URL
    }
}
