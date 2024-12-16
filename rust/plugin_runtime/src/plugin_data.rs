use std::path::PathBuf;
use common::model::PluginId;

pub struct PluginData {
    plugin_id: PluginId,
    plugin_uuid: String,
    plugin_cache_dir: String,
    plugin_data_dir: String,
    inline_view_entrypoint_id: Option<String>,
    home_dir: PathBuf,
}

impl PluginData {
    pub fn new(
        plugin_id: PluginId,
        plugin_uuid: String,
        plugin_cache_dir: String,
        plugin_data_dir: String,
        inline_view_entrypoint_id: Option<String>,
        home_dir: PathBuf,
    ) -> Self {
        Self {
            plugin_id,
            plugin_uuid,
            plugin_cache_dir,
            plugin_data_dir,
            inline_view_entrypoint_id,
            home_dir
        }
    }

    pub fn plugin_id(&self) -> PluginId {
        self.plugin_id.clone()
    }

    pub fn plugin_uuid(&self) -> &str {
        &self.plugin_uuid
    }

    pub fn plugin_cache_dir(&self) -> &str {
        &self.plugin_cache_dir
    }

    pub fn plugin_data_dir(&self) -> &str {
        &self.plugin_data_dir
    }

    pub fn inline_view_entrypoint_id(&self) -> Option<String> {
        self.inline_view_entrypoint_id.clone()
    }

    pub fn home_dir(&self) -> PathBuf {
        self.home_dir.clone()
    }
}