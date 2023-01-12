fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/openvr.cpp")
        .compile("openvr");
    println!("cargo:rerun-if-changed=src/openvr.cpp");
    println!("cargo:rustc-link-lib=openvr_api");
}
