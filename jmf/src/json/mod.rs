mod author;
mod category;
mod image;
mod marker;
mod pack;
mod trail;

pub use self::image::{ImageDescription, ImageSrc, OverlayImage};
pub use author::Author;
pub use category::{Cat, CatTree};
pub use marker::*;
pub use pack::{FullPack, Pack, PackData, PackDescription};
use std::collections::BTreeSet;
pub use trail::{TBinDescription, Trail};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dirty {
    /// ignore the rest of the struct and just save everything (delete the folder first if there's anything left)
    pub pack_desc: bool,
    pub image_desc: bool,
    pub tbin_desc: bool,
    pub string_desc: bool,
    pub cat_desc: bool,
    pub cat_tree: bool,
    /// save the markers from these map ids
    pub markers: BTreeSet<u16>,
    /// same as above, but trails
    pub trails: BTreeSet<u16>,
    /// save the images that have been with these ids, if the image doesn't exist, delete that image instead from filesystem
    pub images: BTreeSet<u16>,
    /// same as above, but for tbins.
    pub tbins: BTreeSet<u16>,
}

impl Dirty {
    pub fn full_from_pack(fp: &FullPack) -> Self {
        let markers: BTreeSet<u16> = fp
            .pack
            .markers
            .keys()
            .copied()
            .map(|id| (id >> 16) as u16)
            .collect();
        let trails: BTreeSet<u16> = fp
            .pack
            .trails
            .keys()
            .copied()
            .map(|id| (id >> 16) as u16)
            .collect();
        let images: BTreeSet<u16> = fp.pack_data.images.keys().copied().collect();
        let tbins: BTreeSet<u16> = fp.pack_data.tbins.keys().copied().collect();
        Self {
            pack_desc: true,
            image_desc: true,
            tbin_desc: true,
            string_desc: true,
            cat_desc: true,
            cat_tree: true,
            markers,
            trails,
            images,
            tbins,
        }
    }
}

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
