fn main() -> Result<(), Box<dyn std::error::Error>> {
    // From CARGO_MANIFEST_DIR of services/core/crates/proto-gen/, going up 4
    // levels (`../../../../proto`) reaches the monorepo `proto/` directory.
    let proto_dir = "../../../../proto";
    // grpc_tools ships the well-known Google proto files (timestamp.proto, etc.)
    // bundled with the `protoc` binary. If a `PROTOC_INCLUDE` env var is set,
    // prefer that (the usual CI / dev path). Otherwise fall back to a hard-coded
    // path that matches the local anaconda grpc-tools install — the build will
    // still succeed because `compile_well_known_types(true)` inlines the WKTs.
    let protoc_include = std::env::var("PROTOC_INCLUDE").ok();
    let mut includes: Vec<String> = vec![proto_dir.to_string()];
    if let Some(inc) = protoc_include {
        includes.push(inc);
    }

    // Walk `proto/` recursively and collect every `*.proto` file.
    let mut protos: Vec<String> = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                walk(&path, out)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("proto") {
                println!("cargo:rerun-if-changed={}", path.display());
                out.push(path.to_str().unwrap().to_string());
            }
        }
        Ok(())
    }
    walk(std::path::Path::new(proto_dir), &mut protos)?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_well_known_types(true)
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR")?)
                .join("market_descriptor.bin"),
        )
        .compile_protos(&protos, &includes)?;

    Ok(())
}
