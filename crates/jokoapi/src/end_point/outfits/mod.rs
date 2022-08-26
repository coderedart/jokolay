use super::{items, EndPoint, EndPointIds};
use url::Url;
type OutfitId = u32;
const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/colors");
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Outfit {
    id: OutfitId,
    name: String,
    icon: Option<Url>,
    unlock_items: Option<Vec<items::ItemId>>,
}

pub struct Outfits;

impl EndPoint for Outfits {
    type RType = Vec<OutfitId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Outfits {
    type Id = OutfitId;
    type RType = Vec<Outfit>;

    fn get_url() -> &'static str {
        E_P_URL
    }
}
