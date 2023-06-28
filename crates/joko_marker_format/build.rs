fn main() {
    cxx_build::bridge("src/lib.rs") // our extern declaration in rust for rapid_filter
        .file("vendor/rapid/rapid.cpp") // our compilation unit containing definition
        .warnings(false)
        .extra_warnings(false)
        .compile("rapid"); // name of library = librapid.a

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=vendor/rapid/rapid.cpp");
    println!("cargo:rerun-if-changed=vendor/rapid/rapid.hpp");
    println!("cargo:rerun-if-changed=vendor/rapid/rapidxml.hpp");
    println!("cargo:rerun-if-changed=vendor/rapid/rapidxml_print.hpp");
    // shadow_rs::new().expect("failed to run shadow");
}
