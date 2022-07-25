use crate::is_default;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Trail {
    /// Alpha to increase the transparency of the trail
    /// if unspecified, will be fully opaque: `255`
    pub alpha: Option<u8>,
    /// category id to which trail belongs to
    /// Validation: Category with this id must exist
    pub cat: Utf8PathBuf,
    /// The color tint to be mixed with the Trail
    /// format is sRGBA
    /// if unspecified, will use `[0, 0, 0, 0]` as fully transparent color to not affect the trail
    pub color: Option<[u8; 4]>,
    /// The name of the png to be used as texture
    /// Validation: should exist in the images/ directory as `name.png` file where `name` is the
    ///     contents of the string.
    /// if empty (default), use the default trail texture.
    #[serde(skip_serializing_if = "is_default")]
    pub texture: String,
    /// refers to the name of tbin to be used as the mesh
    /// Validation: should exist in tbins/ directory as `name.tbin` file where `name` is the contents of the string.
    /// must not be empty
    #[serde(skip_serializing_if = "is_default")]
    pub trl: String,
}
