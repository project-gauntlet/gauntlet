use std::collections::HashMap;
use common::model::{EntrypointId, PhysicalKey};
use crate::model::{AdditionalSearchItem, PreferenceUserData};

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
}