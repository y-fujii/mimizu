fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src/openvr.cpp")
        .file("src/winutils.cpp")
        .compile("capi");
    println!("cargo:rerun-if-changed=src/openvr.cpp");
    println!("cargo:rerun-if-changed=src/winutils.cpp");
    println!("cargo:rustc-link-lib=openvr_api");
}
