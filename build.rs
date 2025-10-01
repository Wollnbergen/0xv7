use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::configure()
        .compile_protos(&["proto/sultan.proto"], &["proto"])?;
    Ok(())
}
