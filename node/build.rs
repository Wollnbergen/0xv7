// sultan-interop/build.rs - Production gRPC build for SDK (full proto compat, eternal)
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let proto_dir = "proto";
    let proto_file = Path::new(proto_dir).join("sultan.proto"); // Relative fix for full proto
    if !proto_file.exists() {
        panic!("proto/sultan.proto missing—add provided full file");
    }
    tonic_build::configure().compile_protos(&[proto_file], &[proto_dir])?; // Renamed, full path
    Ok(())
}
