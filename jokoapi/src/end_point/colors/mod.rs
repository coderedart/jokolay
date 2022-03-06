use super::{EndPoint, EndPointIds};

type ColorId = u32;
const E_P_URL: &str = const_format::concatcp!(crate::API_BASE_V2_URL, "/colors");
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Color {
    id: ColorId,
    name: String,
    base_rgb: [u32; 3],
    cloth: ColorDetailedInfoObject,
    leather: ColorDetailedInfoObject,
    metal: ColorDetailedInfoObject,
    fur: Option<ColorDetailedInfoObject>,
    item: Option<u32>,
    categories: Vec<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ColorDetailedInfoObject {
    brightness: i32,
    contrast: f32,
    hue: u32,
    saturation: f32,
    lightness: f32,
    rgb: [u32; 3],
}

pub struct Colors;

impl EndPoint for Colors {
    type RType = Vec<ColorId>;
    fn get_url() -> &'static str {
        E_P_URL
    }
}

impl EndPointIds for Colors {
    type Id = ColorId;
    type RType = Vec<Color>;

    fn get_url() -> &'static str {
        E_P_URL
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::end_point::{EndPoint, EndPointIds};

    use super::Colors;

    #[tokio::test]
    #[ignore]
    async fn check_color() -> color_eyre::Result<()> {
        let client = Client::new();
        let result = Colors::get(client.clone()).await?;
        assert_eq!(
            result[0],
            Colors::get_with_id(client, &[result[0]]).await?[0].id
        );
        Ok(())
    }
}
