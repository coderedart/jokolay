//! Jokolay Marker Format
//! The library is intended to deal with Json marker packs used by Jokolay.
//!
//! we avoid directly using XML packs because there's no good libraries for XML in Rust.
//! Instead, we convert XML packs into our own format based on Json.
//! we use Json because it has god tier support within the rust ecosystem and it has a bunch of
//! other tiny little useful things like jsondiff / jsonpatch RFCs.
//!
//! The differences between XML packs and JSON packs:
//! 1. XML markers / trails refer to the category they belong to using xpath, and refer to images /
//!     trail binary files using relative path from pack root. JSON markers instead refer to category
//!     by a unique id, and refer to images / trail binary files by their name. this allows changes
//!     to the category menu tree structure without affecting the markers / trails.  
//! 2. there's no file structure in XML packs. any xml file can contain categories, markers, trails..
//!     and images/trail binary files can be anywhere inside the pack.
//!     JSON packs have a cats.json for categories, a maps/ directory containing map_id.json per map
//!     which contains markers / trails from that specific map.
//!     images directory contains images and tbins directory contains trail meshes.
//!     this allows partial loading of a marker pack (only load markers / trails from a specific map).
//!     as well as see diffs to make sure that changes to markers / trails within a map only affect that file.
//! 3. tbin files in XML packs contain a series of points in map (world ) coordinates space.
//!     in JSON packs, we instead save the starting map / world coordinate in Trail and inside tbin,
//!     we only save the points in model space (starting from origin). this allows them to be reused
//!     in various maps / trails. for example, a circular tbin mesh of a particular radius. later,
//!     we can add other transform attributes like scale, rotation etc.. to adjust the radius of the
//!     circular tbin mesh for a particular trail, as well as change its orientation.  
//! 4. In XML packs, categories have a name and their xpath is their unique id. markers / trails have
//!     their own UUID (base64 encoded). In JSON packs, categories have a unique id assigned to them.
//!     markers and trails are stored in a Vec and their position / index is used as their unique id
//!     within the map. vecs are faster and we don't have to worry about clashing ids. this also means
//!     that editing a marker pack means we need to remove the previous "activation save data" as the
//!     ids of the markers / trails might have changed.
//!
//! The documentation for `marker`, `trail`, `tbin`, `image`, `category` and `pack` are in their own
//! respective modules.
//!

extern crate core;

#[cfg(feature = "desktop")]
pub mod bevy;
pub mod manager;
// for compile time build info like pkg version or build timestamp or git hash etc..
shadow_rs::shadow!(build);

// to filter the xml with rapidxml first
#[cxx::bridge(namespace = "rapid")]
mod ffi {

    unsafe extern "C++" {
        include!("jmf/vendor/rapid/rapid.hpp");
        pub fn rapid_filter(src_xml: String) -> String;

    }
}

pub fn rapid_filter_rust(src_xml: String) -> String {
    ffi::rapid_filter(src_xml)
}

pub const INCHES_PER_METER: f32 = 39.370_08;

pub fn is_default<T: PartialEq + Default>(t: &T) -> bool {
    t == &T::default()
}
