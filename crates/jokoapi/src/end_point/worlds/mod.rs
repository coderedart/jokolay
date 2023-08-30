use crate::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct World {
    id: u32,
    name: String,
    population: String,
}
impl EndPoint for World {
    type Id = u32;
    const URL: &'static str = const_format::concatcp!(API_BASE_V2_URL, "/worlds");
    const AUTH: bool = false;
}
