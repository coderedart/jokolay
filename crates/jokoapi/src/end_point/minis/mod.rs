use super::{Deserialize, EndPoint, EndPointIds, Serialize};
use joko_core::prelude::Url;
pub type MiniId = u32;

const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/minis");

#[derive(Serialize, Deserialize)]
pub struct Mini {
    id: MiniId,
    name: String,
    icon: Url,
    order: u32,
    item_id: u32,
}

pub struct Minis;
impl EndPoint for Minis {
    type RType = Vec<MiniId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Minis {
    type Id = MiniId;
    type RType = Vec<Mini>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

// #[cfg(test)]
// mod tests {
//     use reqwest::Client;

//     use crate::end_point::{EndPoint, EndPointIds};

//     use super::Minis;

//     #[test]
//     fn check_mini() {
//         let client = Client::new();
//         let result = Minis::get(client.clone())?();
//         assert_eq!(result[0], Minis::get_with_id(client, &[result[0]])?()[0].id )
//     }
// }
