mod author;
mod category;
mod image;
mod marker;
mod pack;
mod trail;

pub use self::image::{ImageDescription, ImageSrc, OverlayImage};
pub use author::Author;
pub use category::{Cat, CatTree};
pub use marker::{Achievement, Behavior, Dynamic, Info, Marker, MarkerFlags, Trigger};
pub use pack::{FullPack, Pack, PackData, PackDescription};
pub use trail::{TBinDescription, Trail};

//
// use derive_more::*;
//
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut, Display)]
// pub struct Alpha(u8);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut, Display)]
// pub struct CatID(u16);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut)]
// pub struct ClampSize([u16; 2]);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut)]
// pub struct Color([u8; 4]);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut)]
// pub struct FadeRange([f32; 2]);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut, Display)]
// pub struct MapSize(u16);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut, Display)]
// pub struct MapFadeOutScaleLevel(f32);
// #[derive(Debug, Copy, Clone, From, Into, AsRef, AsMut, Display)]
// pub struct Scale(f32);
//
// impl Default for Alpha {
//     fn default() -> Self {
//         Self(255)
//     }
// }
//
// impl Default for CatID {
//     fn default() -> Self {
//         Self(0)
//     }
// }
//
// impl Default for ClampSize {
//     fn default() -> Self {
//         Self([0, u16::MAX])
//     }
// }
//
// impl Default for Color {
//     fn default() -> Self {
//         Self([0, 0, 0, 0])
//     }
// }
//
// impl Default for FadeRange {
//     fn default() -> Self {
//         Self([0.0, 0.0])
//     }
// }
