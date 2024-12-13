use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AdditionalSearchItem {
    pub entrypoint_name: String,
    pub generator_entrypoint_id: String,
    pub entrypoint_id: String,
    pub entrypoint_uuid: String,
    pub entrypoint_icon: Option<Vec<u8>>,
    pub entrypoint_actions: Vec<AdditionalSearchItemAction>,
}

#[derive(Debug, Deserialize)]
pub struct AdditionalSearchItemAction {
    pub id: Option<String>,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PreferenceUserData {
    Number(f64),
    String(String),
    Bool(bool),
    ListOfStrings(Vec<String>),
    ListOfNumbers(Vec<f64>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardData {
    pub text_data: Option<String>,
    pub png_data: Option<Vec<u8>>
}