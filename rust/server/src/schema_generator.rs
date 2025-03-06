mod manifest_models;

use schemars::schema_for;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use manifest_models::models::*;

fn main() {
    let schema = schema_for!(PluginManifest);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    let mut file = File::create("schema.json").expect("Failed to create schema.json");
    file.write_all(json.as_bytes()).expect("Failed to write schema");

    println!("Schema generated and saved to schema.json");
}
