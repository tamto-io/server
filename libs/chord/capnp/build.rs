fn main() -> Result<(), Box<dyn std::error::Error>> {
    capnpc::CompilerCommand::new()
        .file("capnp/chord.capnp")
        .run()?;
    Ok(())
}
