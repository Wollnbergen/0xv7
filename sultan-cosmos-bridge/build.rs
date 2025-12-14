use cbindgen::{Builder, Language};
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("include")
        .join("sultan_bridge.h");

    Builder::new()
        .with_crate(crate_dir)
        .with_language(Language::C)
        .with_cpp_compat(true)
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(output_file);

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi.rs");
}
