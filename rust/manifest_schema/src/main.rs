use std::io::Write;

use gauntlet_server::plugins::plugin_manifest::PluginManifest;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(PluginManifest);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    std::fs::create_dir_all("../../docs/schema").expect("Failed to create directory");
    std::fs::write("../../docs/schema/plugin_manifest.schema.json", json.as_bytes()).expect("Failed to write schema");

    println!("Schema generated and saved to schema.json");
}
