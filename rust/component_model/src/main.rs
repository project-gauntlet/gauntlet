use std::env;
use std::fs;

use anyhow::anyhow;
use gauntlet_component_model::create_component_model;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let path_to_save = &args[1];

            let components = create_component_model();

            let json = serde_json::to_string_pretty(&components)?;

            fs::write(path_to_save, json)?;

            Ok(())
        }
        args @ _ => Err(anyhow!("Unsupported args number: {args}")),
    }
}
