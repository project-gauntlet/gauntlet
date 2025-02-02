use vergen_gitcl::CargoBuilder;
use vergen_gitcl::Emitter;
use vergen_gitcl::GitclBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=db_migrations");

    let gitcl = GitclBuilder::all_git()?;
    let cargo = CargoBuilder::default()
        .debug(true)
        .features(true)
        .opt_level(true)
        .target_triple(true)
        .build()?;

    Emitter::default()
        .add_instructions(&gitcl)?
        .add_instructions(&cargo)?
        .emit()?;

    Ok(())
}
