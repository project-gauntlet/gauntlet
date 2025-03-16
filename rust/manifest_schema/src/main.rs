use std::io::Write;
use std::path::PathBuf;

use gauntlet_server::plugins::plugin_manifest::PluginManifest;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(PluginManifest);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    let schema_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "../../../docs/schema/plugin_manifest.schema.json"
    ));

    std::fs::create_dir_all(schema_path.parent().unwrap()).expect("Failed to create directory");
    std::fs::write(schema_path, json.as_bytes()).expect("Failed to write schema");

    println!("Schema generated and saved to schema.json");
}
