use super::{EndPoint, EndPointIds};
use serde::{Deserialize, Serialize};
use url::Url;

pub type QuagganId = String;
const E_P_URL: &str = const_format::concatcp!(crate::jokoapi::API_BASE_V2_URL, "/quaggans");

#[derive(Serialize, Deserialize)]
pub struct Quaggan {
    id: QuagganId,
    url: Url,
}

pub struct Quaggans;
impl EndPoint for Quaggans {
    type RType = Vec<QuagganId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Quaggans {
    type Id = QuagganId;
    type RType = Vec<Quaggan>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}
