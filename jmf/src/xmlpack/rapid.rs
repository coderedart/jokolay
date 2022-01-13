#[cxx::bridge(namespace = "rapid")]
mod ffi {

    unsafe extern "C++" {
        include!("jmf/rapid/rapid.hpp");
        fn rapid_filter(src_xml: String) -> String;

    }
}

pub use ffi::rapid_filter;
// pub fn rapid_filter_rust(src_xml: String) -> String {
//     ffi::rapid_filter(src_xml)
// }