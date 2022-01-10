#[cxx::bridge(namespace = "rapid")]
mod ffi {

    unsafe extern "C++" {
        include!("jmf/rapid/rapid.hpp");
        fn rapid_filter(src_xml: String) -> String;

    }
}

pub fn rapid_filter_(src_xml: String) -> String {
    ffi::rapid_filter(src_xml)
}