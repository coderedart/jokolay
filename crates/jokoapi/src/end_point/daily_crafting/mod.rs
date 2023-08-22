use super::{Deserialize, EndPoint, EndPointIds, Serialize, EndPointId};

type DailycraftingRecipeId = String;
const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/dailycrafting");
#[derive(Serialize, Deserialize)]
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
impl EndPointId for Dailycrafting {
    type RType = DailycraftingRecipe;

    type Id = DailycraftingRecipeId;

    fn get_url(id: &Self::Id) -> &'static str {
        E_P_URL
    }
}