fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iface_files = &["./proto/helloworld.proto"];
    let dirs = &["./proto"];

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(iface_files, dirs)?;

    for file in iface_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    Ok(())
}
