use super::{Deserialize, EndPoint, Serialize};

#[derive(Serialize, Deserialize)]
pub struct World {
    id: u32,
    name: String,
    population: String,
}
impl EndPoint for World {
    type Id = u32;
    const URL: &'static str = const_format::concatcp!(crate::API_BASE_V2_URL, "/worlds");
    const AUTH: bool = false;
}
