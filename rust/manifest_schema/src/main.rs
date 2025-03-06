use gauntlet_server::plugins::plugin_manifest_models::models::PluginManifest;

use schemars::schema_for;
use std::fs::File;
use std::io::Write;

fn main() {
    let schema = schema_for!(PluginManifest);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    let mut file = File::create("plugin_manifest.schema.json").expect("Failed to create schema.json");
    file.write_all(json.as_bytes()).expect("Failed to write schema");

    println!("Schema generated and saved to schema.json");
}
