//! ReadOnly XML marker packs support for Jokolay
//!
//!

pub mod manager;
pub mod pack;
// for compile time build info like pkg version or build timestamp or git hash etc..
// shadow_rs::shadow!(build);

// to filter the xml with rapidxml first
#[cxx::bridge(namespace = "rapid")]
mod ffi {
    unsafe extern "C++" {
        include!("joko_marker_format/vendor/rapid/rapid.hpp");
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

mod temp {

    pub trait MarkerManagerBroker {
        /*
        It is hard to find a format which will support both editing and efficient reading. So, we will have a readonly format stored on disk.
        And a write only version in memory when we want to edit a pack.

        We will continue with the current rkyv based format for readonly version. Although it is huge, it can be mmapped.

        receive a map/char changed event to trigger markers loading
        get current map categories + markers + trails + textures of each installed/enabled readonly pack. This is the only moment when you want to read from the file.

        load activation stats of categories + markers + trails
        filter markers + trails + textures from enabled categories
        filter markers + trails based on character race/class and other static data


         */
    }
}
