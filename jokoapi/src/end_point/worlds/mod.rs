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

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::end_point::{EndPoint, EndPointIds};

    use super::Worlds;

    #[tokio::test]
    async fn check_world() {
        let client = Client::new();
        let result = Worlds::get(client.clone()).await.unwrap();
        assert_eq!(
            result[0],
            Worlds::get_with_id(client, &[result[0]]).await.unwrap()[0].id
        )
    }
}
