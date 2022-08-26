use super::{EndPoint, EndPointIds};

type WorldId = u32;
const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/worlds");
#[derive(serde::Serialize, serde::Deserialize)]
pub struct World {
    id: u32,
    name: String,
    population: String,
}

pub struct Worlds;

impl EndPoint for Worlds {
    type RType = Vec<WorldId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Worlds {
    type Id = WorldId;
    type RType = Vec<World>;

    fn get_url() -> &'static str {
        E_P_URL
    }
}
