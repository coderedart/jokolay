use super::{EndPoint, EndPointIds};

type DailycraftingRecipeId = String;
const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/dailycrafting");
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DailycraftingRecipe {
    id: DailycraftingRecipeId,
}
pub struct Dailycrafting;

impl EndPoint for Dailycrafting {
    type RType = Vec<DailycraftingRecipeId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Dailycrafting {
    type Id = DailycraftingRecipeId;
    type RType = Vec<DailycraftingRecipe>;

    fn get_url() -> &'static str {
        E_P_URL
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::end_point::{EndPoint, EndPointIds};

    use super::Dailycrafting;

    #[tokio::test]
    #[ignore]
    async fn check_dailycrafting() -> color_eyre::Result<()> {
        let client = Client::new();
        let result = Dailycrafting::get(client.clone()).await?;
        assert_eq!(
            result[0],
            Dailycrafting::get_with_id(client, &[result[0].clone()]).await?[0].id
        );

        Ok(())
    }
}
