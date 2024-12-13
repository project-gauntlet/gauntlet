use crate::model::{AdditionalSearchItem, ClipboardData, PreferenceUserData};
use common::model::{EntrypointId, PhysicalKey};
use std::collections::HashMap;

pub trait BackendForPluginRuntimeApi {
    async fn reload_search_index(&self, generated_commands: Vec<AdditionalSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> ;
    async fn get_asset_data(&self, path: &str) -> anyhow::Result<Vec<u8>>;
    async fn get_command_generator_entrypoint_ids(&self) -> anyhow::Result<Vec<String>>;
    async fn get_action_id_for_shortcut(
        &self,
        entrypoint_id: &str,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<Option<String>>;
    async fn get_plugin_preferences(&self) -> anyhow::Result<HashMap<String, PreferenceUserData>>;
    async fn get_entrypoint_preferences(&self, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, PreferenceUserData>>;
    async fn plugin_preferences_required(&self) -> anyhow::Result<bool>;
    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> anyhow::Result<bool>;
    async fn clipboard_read(&self) -> anyhow::Result<ClipboardData>;
    async fn clipboard_read_text(&self) -> anyhow::Result<Option<String>>;
    async fn clipboard_write(&self, data: ClipboardData) -> anyhow::Result<()>;
    async fn clipboard_write_text(&self, data: String) -> anyhow::Result<()>;
    async fn clipboard_clear(&self) -> anyhow::Result<()>;
}