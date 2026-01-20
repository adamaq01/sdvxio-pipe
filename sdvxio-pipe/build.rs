use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .clang_arg("-I./include")
        .header("include/bemanitools/glue.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate glue bindings")
        .write_to_file(out_path.join("glue.rs"))
        .expect("Couldn't write glue bindings!");
}
