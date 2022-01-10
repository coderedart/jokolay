fn main() {
    cxx_build::bridge("src/xmlpack/rapid.rs")  // returns a cc::Build
    .file("rapid/rapid.cpp")
    .flag_if_supported("-std=c++14")
    .flag_if_supported("-fpermissive")
    .compile("rapid");

println!("cargo:rerun-if-changed=src/xmlpack/rapid.rs");
println!("cargo:rerun-if-changed=rapid/rapid.cpp");
println!("cargo:rerun-if-changed=rapid/rapid.hpp");
println!("cargo:rerun-if-changed=rapid/rapidxml.hpp");
println!("cargo:rerun-if-changed=rapid/rapidxml_print.hpp");

}