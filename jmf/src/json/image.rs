use crate::is_default;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ImageDescription {
    #[serde(skip_serializing_if = "is_default")]
    pub name: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "is_default")]
    pub source: ImageSrc,
    #[serde(skip_serializing_if = "is_default")]
    pub credit: String,
    #[serde(skip_serializing_if = "is_default")]
    pub extra: String,
}

impl Default for ImageDescription {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            width: 64,
            height: 64,
            source: Default::default(),
            credit: "".to_string(),
            extra: "".to_string(),
        }
    }
}

impl ImageDescription {
    pub fn from_img_and_src(src: ImageSrc, img: &image::DynamicImage) -> Self {
        Self {
            width: img.width(),
            height: img.height(),
            source: src,
            ..Default::default()
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum ImageSrc {
    FS,
    Url(String),
    OverlayImage(OverlayImage),
}

impl Default for ImageSrc {
    fn default() -> Self {
        ImageSrc::OverlayImage(OverlayImage::default())
    }
}

impl ImageSrc {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum OverlayImage {
    Marker,
    Trail,
    Unknown,
    Loading,
}

impl Default for OverlayImage {
    fn default() -> Self {
        OverlayImage::Marker
    }
}

#[cfg(test)]
mod test {
    use crate::json::image::{ImageDescription, ImageSrc};
    use serde_test::{assert_tokens, Token};

    #[test]
    fn serde_image_descriptions() {
        let idesc = ImageDescription {
            name: "marker".to_string(),
            width: 128,
            height: 128,
            source: ImageSrc::FS,
            credit: "Joko".to_string(),
            extra: "".to_string(),
        };

        assert_tokens(
            &idesc,
            &[
                Token::Struct {
                    name: "ImageDescription",
                    len: 5,
                },
                Token::Str("name"),
                Token::String("marker"),
                Token::Str("width"),
                Token::U32(128),
                Token::Str("height"),
                Token::U32(128),
                Token::Str("source"),
                Token::Enum { name: "ImageSrc" },
                Token::Str("FS"),
                Token::Unit,
                Token::Str("credit"),
                Token::String("Joko"),
                Token::StructEnd,
            ],
        );
    }
}
