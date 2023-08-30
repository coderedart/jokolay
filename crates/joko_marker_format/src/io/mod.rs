//! This modules primarily deals with serializing and deserializing xml data from marker packs
//!

use xot::{NameId, Xot};

mod deserialize;
mod error;
mod serialize;

pub use deserialize::{get_pack_from_taco_zip, load_pack_core_from_dir};
pub use serialize::save_pack_core_to_dir;
struct XotAttributeNameIDs {
    pub overlay_data: NameId,
    pub marker_category: NameId,
    pub pois: NameId,
    pub poi: NameId,
    pub trail: NameId,
    pub category: NameId,
    pub xpos: NameId,
    pub ypos: NameId,
    pub zpos: NameId,
    pub icon_file: NameId,
    pub texture: NameId,
    pub trail_data: NameId,
    pub separator: NameId,
    pub display_name: NameId,
    pub default_enabled: NameId,
    pub name: NameId,
    pub map_id: NameId,
    pub guid: NameId,
}
impl XotAttributeNameIDs {
    pub fn register_with_xot(tree: &mut Xot) -> Self {
        Self {
            overlay_data: tree.add_name("OverlayData"),
            marker_category: tree.add_name("MarkerCategory"),
            pois: tree.add_name("POIs"),
            poi: tree.add_name("POI"),
            trail: tree.add_name("Trail"),
            category: tree.add_name("type"),
            xpos: tree.add_name("xpos"),
            ypos: tree.add_name("ypos"),
            zpos: tree.add_name("zpos"),
            icon_file: tree.add_name("iconFile"),
            texture: tree.add_name("texture"),
            trail_data: tree.add_name("trailData"),
            separator: tree.add_name("IsSeparator"),
            name: tree.add_name("name"),
            default_enabled: tree.add_name("defaulttoggle"),
            display_name: tree.add_name("DisplayName"),
            map_id: tree.add_name("MapID"),
            guid: tree.add_name("GUID"),
        }
    }
}
