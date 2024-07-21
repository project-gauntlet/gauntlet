fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &["./../../schema/backend.proto"],
            &["./../../schema/"],
        )?;

    Ok(())
}
