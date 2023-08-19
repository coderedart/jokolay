use super::{EndPoint, EndPointIds};

type ColorId = u32;
const E_P_URL: &str = const_format::concatcp!(crate::jokoapi::API_BASE_V2_URL, "/colors");
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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
