use bindgen::Abi;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=include/wrapper.h");

    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "x86_64" {
        println!("cargo:rustc-link-search=libs");
    } else {
        println!("cargo:rustc-link-search=libs/x86");
    };
    println!("cargo:rustc-link-lib=dylib=sdvxio");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .clang_arg("-I./include")
        .header("include/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .override_abi(Abi::System, "sdvxio")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
