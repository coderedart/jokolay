use super::{EndPoint, EndPointIds};
use serde::{Deserialize, Serialize};
use url::Url;

pub type QuagganId = String;
const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/quaggans");

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

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::end_point::{EndPoint, EndPointIds};

    use super::Quaggans;

    #[tokio::test]
    #[ignore]
    async fn check_quaggan() {
        let client = Client::new();
        let result = Quaggans::get(client.clone()).await.unwrap();
        assert_eq!(
            result[0],
            Quaggans::get_with_id(client, &[result[0].clone()])
                .await
                .unwrap()[0]
                .id
        );
    }
}
