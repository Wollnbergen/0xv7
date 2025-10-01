use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Use workspace root to resolve proto path robustly
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let proto_file = format!("{}/../proto/sultan.proto", manifest_dir);
    let proto_dir = format!("{}/../proto", manifest_dir);
    tonic_build::configure()
        .compile(&[&proto_file], &[&proto_dir])?;
    Ok(())
}
