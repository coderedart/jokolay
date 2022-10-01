//! ReadOnly XML marker packs support for Jokolay
//!
//!

pub mod manager;
pub mod pack;
pub mod render;
// for compile time build info like pkg version or build timestamp or git hash etc..
// shadow_rs::shadow!(build);

// to filter the xml with rapidxml first
#[cxx::bridge(namespace = "rapid")]
mod ffi {

    unsafe extern "C++" {
        include!("jokolay/vendor/rapid/rapid.hpp");
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
