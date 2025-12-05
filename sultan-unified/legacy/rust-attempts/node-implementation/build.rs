fn main() {
    // Skip protobuf for now to get a clean build
    println!("cargo:rerun-if-changed=build.rs");
}
