use deno_core::JsBuffer;
use deno_core::ToJsBuffer;
use gauntlet_common_plugin_runtime::model::JsGeneratedSearchItemAccessory;
use gauntlet_common_plugin_runtime::model::JsGeneratedSearchItemAction;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize)]
pub struct DenoOutGeneratedSearchItem {
    pub entrypoint_name: String,
    pub generator_entrypoint_id: String,
    pub entrypoint_id: String,
    pub entrypoint_uuid: String,
    pub entrypoint_icon: Option<ToJsBuffer>,
    pub entrypoint_actions: Vec<JsGeneratedSearchItemAction>,
    pub entrypoint_accessories: Vec<JsGeneratedSearchItemAccessory>,
}

#[derive(Deserialize)]
pub struct DenoInGeneratedSearchItem {
    pub entrypoint_name: String,
    pub generator_entrypoint_id: String,
    pub entrypoint_id: String,
    pub entrypoint_uuid: String,
    pub entrypoint_icon: Option<JsBuffer>,
    pub entrypoint_actions: Vec<JsGeneratedSearchItemAction>,
    pub entrypoint_accessories: Vec<JsGeneratedSearchItemAccessory>,
}

#[derive(Serialize)]
pub struct DenoOutClipboardData {
    pub text_data: Option<String>,
    pub png_data: Option<ToJsBuffer>,
}

#[derive(Deserialize)]
pub struct DenoInClipboardData {
    pub text_data: Option<String>,
    pub png_data: Option<JsBuffer>,
}
