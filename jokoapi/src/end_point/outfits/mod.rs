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

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::end_point::{EndPoint, EndPointIds};

    use super::Outfits;

    #[tokio::test]
    async fn check_outfit() -> anyhow::Result<()> {
        let client = Client::new();
        let result = Outfits::get(client.clone()).await?;
        assert_eq!(
            result[0],
            Outfits::get_with_id(client, &[result[0]]).await?[0].id
        );
        Ok(())
    }
}
