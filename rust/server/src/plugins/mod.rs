use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::DownloadStatus;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::KeyboardEventOrigin;
use gauntlet_common::model::LocalSaveData;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::PluginPreference;
use gauntlet_common::model::PluginPreferenceUserData;
use gauntlet_common::model::PreferenceEnumValue;
use gauntlet_common::model::SearchResult;
use gauntlet_common::model::SearchResultEntrypointType;
use gauntlet_common::model::SettingsEntrypoint;
use gauntlet_common::model::SettingsEntrypointType;
use gauntlet_common::model::SettingsPlugin;
use gauntlet_common::model::SettingsTheme;
use gauntlet_common::model::UiPropertyValue;
use gauntlet_common::model::UiRequestData;
use gauntlet_common::model::UiResponseData;
use gauntlet_common::model::UiSetupData;
use gauntlet_common::model::UiWidgetId;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::rpc::frontend_api::FrontendApi;
use gauntlet_common::settings_env_data_to_string;
use gauntlet_common::SettingsEnvData;
use gauntlet_common::SETTINGS_ENV;
use gauntlet_plugin_runtime::JsPluginCode;
use gauntlet_plugin_runtime::JsPluginPermissions;
use gauntlet_plugin_runtime::JsPluginPermissionsExec;
use gauntlet_plugin_runtime::JsPluginPermissionsFileSystem;
use gauntlet_plugin_runtime::JsPluginPermissionsMainSearchBar;
use gauntlet_utils::channel::RequestSender;
use include_dir::include_dir;
use include_dir::Dir;
use tokio::runtime::Handle;

use crate::model::ActionShortcutKey;
use crate::plugins::clipboard::Clipboard;
use crate::plugins::config_reader::ConfigReader;
use crate::plugins::data_db_repository::db_entrypoint_from_str;
use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::data_db_repository::DbPluginActionShortcutKind;
use crate::plugins::data_db_repository::DbPluginClipboardPermissions;
use crate::plugins::data_db_repository::DbPluginEntrypointType;
use crate::plugins::data_db_repository::DbPluginMainSearchBarPermissions;
use crate::plugins::data_db_repository::DbPluginPreference;
use crate::plugins::data_db_repository::DbPluginPreferenceUserData;
use crate::plugins::data_db_repository::DbReadPluginEntrypoint;
use crate::plugins::icon_cache::IconCache;
use crate::plugins::js::start_plugin_runtime;
use crate::plugins::js::AllPluginCommandData;
use crate::plugins::js::OnePluginCommandData;
use crate::plugins::js::PluginCommand;
use crate::plugins::js::PluginPermissions;
use crate::plugins::js::PluginPermissionsClipboard;
use crate::plugins::js::PluginRuntimeData;
use crate::plugins::loader::PluginLoader;
use crate::plugins::run_status::RunStatusHolder;
use crate::plugins::settings::Settings;
use crate::search::EntrypointActionDataView;
use crate::search::EntrypointActionType;
use crate::search::EntrypointDataView;
use crate::search::PluginDataView;
use crate::search::SearchIndex;

mod clipboard;
mod config_reader;
mod data_db_repository;
mod download_status;
pub(super) mod frecency;
mod icon_cache;
mod image_gatherer;
pub mod js;
mod loader;
mod run_status;
mod runtime;
mod settings;
mod theme;

static BUNDLED_PLUGINS: [(&str, Dir); 1] = [(
    "gauntlet",
    include_dir!("$CARGO_MANIFEST_DIR/../../bundled_plugins/gauntlet/dist"),
)];

pub struct ApplicationManager {
    config_reader: ConfigReader,
    search_index: SearchIndex,
    command_broadcaster: tokio::sync::broadcast::Sender<PluginCommand>,
    db_repository: DataDbRepository,
    plugin_downloader: PluginLoader,
    run_status_holder: RunStatusHolder,
    icon_cache: IconCache,
    frontend_api: FrontendApi,
    dirs: Dirs,
    clipboard: Clipboard,
    settings: Settings,
}

impl ApplicationManager {
    pub async fn create(frontend_sender: RequestSender<UiRequestData, UiResponseData>) -> anyhow::Result<Self> {
        let frontend_api = FrontendApi::new(frontend_sender);
        let dirs = Dirs::new();
        let db_repository = DataDbRepository::new(dirs.clone()).await?;
        let plugin_downloader = PluginLoader::new(db_repository.clone());
        let config_reader = ConfigReader::new(dirs.clone(), db_repository.clone());
        let icon_cache = IconCache::new(dirs.clone());
        let run_status_holder = RunStatusHolder::new();
        let search_index = SearchIndex::create_index(frontend_api.clone())?;
        let clipboard = Clipboard::new()?;
        let settings = Settings::new(dirs.clone(), db_repository.clone(), frontend_api.clone())?;

        let (command_broadcaster, _) = tokio::sync::broadcast::channel::<PluginCommand>(100);

        Ok(Self {
            config_reader,
            search_index,
            command_broadcaster,
            db_repository,
            plugin_downloader,
            run_status_holder,
            icon_cache,
            frontend_api,
            clipboard,
            settings,
            dirs,
        })
    }

    pub async fn setup_data(&self) -> anyhow::Result<UiSetupData> {
        let window_position_file = self.dirs.window_position();
        let theme = self.settings.effective_theme().await?;
        let global_shortcut = self.settings.effective_global_shortcut().await?;
        let window_position_mode = self.settings.window_position_mode_setting().await?;
        let close_on_unfocus = self.config_reader.close_on_unfocus();

        Ok(UiSetupData {
            window_position_file,
            theme,
            global_shortcut,
            close_on_unfocus,
            window_position_mode,
        })
    }

    pub async fn setup_response(&self, global_shortcut_error: Option<String>) -> anyhow::Result<()> {
        self.settings.set_global_shortcut_error(global_shortcut_error).await?;

        Ok(())
    }

    pub fn clear_all_icon_cache_dir(&self) -> anyhow::Result<()> {
        tracing::debug!("clearing all icon cache");

        self.icon_cache.clear_all_icon_cache_dir()
    }

    pub async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        self.plugin_downloader.download_plugin(plugin_id).await
    }

    pub fn download_status(&self) -> HashMap<PluginId, DownloadStatus> {
        self.plugin_downloader.download_status()
    }

    pub fn search(&self, text: &str, render_inline_view: bool) -> anyhow::Result<Vec<SearchResult>> {
        let result = self.search_index.search(&text);

        if render_inline_view {
            self.handle_inline_view(&text);
        }

        result
    }

    pub async fn show_window(&self) -> anyhow::Result<()> {
        self.frontend_api.show_window().await?;

        Ok(())
    }

    pub async fn run_action(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_id: String,
    ) -> anyhow::Result<()> {
        let data = self.search_index.plugin_entrypoint_actions();

        let Some(data) = data.get(&plugin_id) else {
            return Err(anyhow!("Unable to find plugin with id: {}", plugin_id));
        };

        let PluginDataView {
            plugin_name,
            entrypoints,
        } = data;

        let Some(entrypoint_data) = &entrypoints.get(&entrypoint_id) else {
            return Err(anyhow!("Unable to find entrypoint with id: {}", entrypoint_id));
        };

        let EntrypointDataView {
            entrypoint_name,
            entrypoint_type,
            actions,
            ..
        } = entrypoint_data;

        match action_id.as_str() {
            ":primary" => {
                match entrypoint_type {
                    SearchResultEntrypointType::Command => {
                        self.handle_run_command(plugin_id, entrypoint_id).await;
                    }
                    SearchResultEntrypointType::View => {
                        self.frontend_api
                            .open_plugin_view(
                                plugin_id,
                                plugin_name.to_string(),
                                entrypoint_id,
                                entrypoint_name.to_string(),
                            )
                            .await?;
                    }
                    SearchResultEntrypointType::Generated => {
                        let Some(action_data) = actions.get(0) else {
                            return Err(anyhow!("Requested entrypoint doesn't provide primary action"));
                        };

                        let EntrypointActionDataView { action_type, .. } = action_data;

                        match action_type {
                            EntrypointActionType::Command => {
                                self.handle_run_generated_entrypoint(plugin_id, entrypoint_id, 0).await;
                            }
                            EntrypointActionType::View => {
                                self.frontend_api
                                    .open_generated_plugin_view(
                                        plugin_id,
                                        plugin_name.to_string(),
                                        entrypoint_id,
                                        entrypoint_name.to_string(),
                                        0,
                                    )
                                    .await?;
                            }
                        }
                    }
                }
            }
            ":secondary" => {
                match entrypoint_type {
                    SearchResultEntrypointType::Command => {
                        return Err(anyhow!("Command entrypoints support only ':primary' action"));
                    }
                    SearchResultEntrypointType::View => {
                        return Err(anyhow!("View entrypoints support only ':primary' action"));
                    }
                    SearchResultEntrypointType::Generated => {
                        let Some(action_data) = actions.get(1) else {
                            return Err(anyhow!("Requested entrypoint doesn't provide secondary action"));
                        };

                        let EntrypointActionDataView { action_type, .. } = action_data;

                        match action_type {
                            EntrypointActionType::Command => {
                                self.handle_run_generated_entrypoint(plugin_id, entrypoint_id, 1).await;
                            }
                            EntrypointActionType::View => {
                                self.frontend_api
                                    .open_generated_plugin_view(
                                        plugin_id,
                                        plugin_name.to_string(),
                                        entrypoint_id,
                                        entrypoint_name.to_string(),
                                        1,
                                    )
                                    .await?;
                            }
                        }
                    }
                }
            }
            action_id @ _ => {
                match entrypoint_type {
                    SearchResultEntrypointType::Command => {
                        return Err(anyhow!("Command entrypoints support only ':primary' action"));
                    }
                    SearchResultEntrypointType::View => {
                        return Err(anyhow!("View entrypoints support only ':primary' action"));
                    }
                    SearchResultEntrypointType::Generated => {
                        let index = entrypoint_data
                            .actions
                            .iter()
                            .position(|data| &data.id == &Some(action_id.to_string()));

                        match index {
                            None => {
                                return Err(anyhow!(
                                    "Requested entrypoint doesn't provide action with id: {}",
                                    action_id
                                ));
                            }
                            Some(index) => {
                                let action_data = &entrypoint_data.actions[index];

                                match action_data.action_type {
                                    EntrypointActionType::Command => {
                                        self.handle_run_generated_entrypoint(plugin_id, entrypoint_id, index)
                                            .await;
                                    }
                                    EntrypointActionType::View => {
                                        self.frontend_api
                                            .open_generated_plugin_view(
                                                plugin_id,
                                                plugin_name.to_string(),
                                                entrypoint_id,
                                                entrypoint_name.to_string(),
                                                index,
                                            )
                                            .await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn save_local_plugin(&self, path: &str) -> anyhow::Result<LocalSaveData> {
        tracing::info!(target = "plugin", "Saving local plugin at path: {:?}", path);

        let plugin_id = self.plugin_downloader.save_local_plugin(path).await?;

        let plugin = self.db_repository.get_plugin_by_id(&plugin_id.to_string()).await?;

        self.reload_plugin(plugin_id.clone()).await?;

        let (stdout_file_path, stderr_file_path) = self.dirs.plugin_log_files(&plugin.uuid);

        Ok(LocalSaveData {
            stdout_file_path: stdout_file_path
                .into_os_string()
                .into_string()
                .map_err(|_| anyhow!("non uft8 paths are not supported"))?,
            stderr_file_path: stderr_file_path
                .into_os_string()
                .into_string()
                .map_err(|_| anyhow!("non uft8 paths are not supported"))?,
        })
    }

    pub async fn load_bundled_plugins(&self) -> anyhow::Result<()> {
        for (id, dir) in &BUNDLED_PLUGINS {
            tracing::info!(target = "plugin", "Saving builtin plugin with id: {:?}", id);

            let plugin_id = self.plugin_downloader.save_bundled_plugin(id, dir).await?;

            self.reload_plugin(plugin_id).await?;
        }

        Ok(())
    }

    pub async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>> {
        let result = self
            .db_repository
            .list_plugins_and_entrypoints()
            .await?
            .into_iter()
            .map(|(plugin, entrypoints)| {
                let entrypoints = entrypoints
                    .into_iter()
                    .map(|entrypoint| {
                        let entrypoint_id = EntrypointId::from_string(entrypoint.id);

                        let entrypoint = SettingsEntrypoint {
                            enabled: entrypoint.enabled,
                            entrypoint_id: entrypoint_id.clone(),
                            entrypoint_name: entrypoint.name,
                            entrypoint_description: entrypoint.description,
                            entrypoint_type: match db_entrypoint_from_str(&entrypoint.entrypoint_type) {
                                DbPluginEntrypointType::Command => SettingsEntrypointType::Command,
                                DbPluginEntrypointType::View => SettingsEntrypointType::View,
                                DbPluginEntrypointType::InlineView => SettingsEntrypointType::InlineView,
                                DbPluginEntrypointType::EntrypointGenerator => {
                                    SettingsEntrypointType::EntrypointGenerator
                                }
                            }
                            .into(),
                            preferences: entrypoint
                                .preferences
                                .into_iter()
                                .map(|(key, value)| {
                                    let preference = plugin_preference_from_db(&key, value);
                                    (key, preference)
                                })
                                .collect(),
                            preferences_user_data: entrypoint
                                .preferences_user_data
                                .into_iter()
                                .map(|(key, value)| (key, plugin_preference_user_data_from_db(value)))
                                .collect(),
                        };

                        (entrypoint_id, entrypoint)
                    })
                    .collect();

                SettingsPlugin {
                    plugin_id: PluginId::from_string(plugin.id),
                    plugin_name: plugin.name,
                    plugin_description: plugin.description,
                    enabled: plugin.enabled,
                    entrypoints,
                    preferences: plugin
                        .preferences
                        .into_iter()
                        .map(|(key, value)| {
                            let preference = plugin_preference_from_db(&key, value);
                            (key, preference)
                        })
                        .collect(),
                    preferences_user_data: plugin
                        .preferences_user_data
                        .into_iter()
                        .map(|(key, value)| (key, plugin_preference_user_data_from_db(value)))
                        .collect(),
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn set_plugin_state(&self, plugin_id: PluginId, set_enabled: bool) -> anyhow::Result<()> {
        let currently_running = self.run_status_holder.is_plugin_running(&plugin_id);
        let currently_enabled = self.is_plugin_enabled(&plugin_id).await?;

        tracing::info!(
            target = "plugin",
            "Setting plugin state for plugin id: {:?}, currently_running: {}, currently_enabled: {}, set_enabled: {}",
            plugin_id,
            currently_running,
            currently_enabled,
            set_enabled
        );

        match (currently_running, currently_enabled, set_enabled) {
            (false, false, true) => {
                self.db_repository
                    .set_plugin_enabled(&plugin_id.to_string(), true)
                    .await?;

                self.start_plugin(plugin_id).await?;
            }
            (false, true, true) => {
                self.start_plugin(plugin_id).await?;
            }
            (true, true, false) => {
                self.db_repository
                    .set_plugin_enabled(&plugin_id.to_string(), false)
                    .await?;

                self.stop_plugin(plugin_id.clone()).await;
                self.search_index.remove_for_plugin(plugin_id)?;
            }
            (true, false, _) => {
                tracing::error!(
                    "Plugin is running but is disabled, please report this: {}",
                    plugin_id.to_string()
                )
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn set_entrypoint_state(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        enabled: bool,
    ) -> anyhow::Result<()> {
        tracing::debug!(
            target = "plugin",
            "Setting entrypoint state for plugin id: {:?}, entrypoint_id: {:?}, enabled: {}",
            plugin_id,
            entrypoint_id,
            enabled
        );

        self.db_repository
            .set_plugin_entrypoint_enabled(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled)
            .await?;

        self.reload_plugin(plugin_id.clone()).await?;

        Ok(())
    }

    pub async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> anyhow::Result<()> {
        self.settings.set_global_shortcut(shortcut).await
    }

    pub async fn get_global_shortcut(&self) -> anyhow::Result<Option<(Option<PhysicalShortcut>, Option<String>)>> {
        self.settings.global_shortcut().await
    }

    pub async fn set_theme(&self, theme: SettingsTheme) -> anyhow::Result<()> {
        self.settings.set_theme_setting(theme).await
    }

    pub async fn get_theme(&self) -> anyhow::Result<SettingsTheme> {
        self.settings.theme_setting().await
    }

    pub async fn set_window_position_mode(&self, mode: WindowPositionMode) -> anyhow::Result<()> {
        self.settings.set_window_position_mode_setting(mode).await
    }

    pub async fn get_window_position_mode(&self) -> anyhow::Result<WindowPositionMode> {
        self.settings.window_position_mode_setting().await
    }

    pub async fn set_preference_value(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        preference_id: String,
        preference_value: PluginPreferenceUserData,
    ) -> anyhow::Result<()> {
        tracing::debug!(
            target = "plugin",
            "Setting preference value for plugin id: {:?}, entrypoint_id: {:?}, preference_id: {}",
            plugin_id,
            entrypoint_id,
            preference_id
        );

        let user_data = plugin_preference_user_data_to_db(preference_value);

        self.db_repository
            .set_preference_value(
                plugin_id.to_string(),
                entrypoint_id.map(|id| id.to_string()),
                preference_id,
                user_data,
            )
            .await?;

        self.reload_plugin(plugin_id.clone()).await?;

        Ok(())
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        self.config_reader.reload_config().await?;

        Ok(())
    }

    pub async fn reload_all_plugins(&self) -> anyhow::Result<()> {
        tracing::info!("Reloading all plugins");

        self.reload_config().await?;

        for plugin in self.db_repository.list_plugins().await? {
            let plugin_id = PluginId::from_string(plugin.id);
            let running = self.run_status_holder.is_plugin_running(&plugin_id);
            match (running, plugin.enabled) {
                (false, true) => {
                    self.start_plugin(plugin_id).await?;
                }
                (true, false) => {
                    self.stop_plugin(plugin_id.clone()).await;
                    self.search_index.remove_for_plugin(plugin_id)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn remove_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Removing plugin with id: {:?}", plugin_id);

        let running = self.run_status_holder.is_plugin_running(&plugin_id);
        if running {
            self.stop_plugin(plugin_id.clone()).await;
        }
        self.db_repository.remove_plugin(&plugin_id.to_string()).await?;
        self.search_index.remove_for_plugin(plugin_id)?;
        Ok(())
    }

    pub fn handle_inline_view(&self, text: &str) {
        self.send_command(PluginCommand::All {
            data: AllPluginCommandData::OpenInlineView { text: text.to_owned() },
        })
    }

    pub async fn handle_run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) {
        self.send_command(PluginCommand::One {
            id: plugin_id.clone(),
            data: OnePluginCommandData::RunCommand {
                entrypoint_id: entrypoint_id.to_string(),
            },
        });

        self.mark_entrypoint_frecency(plugin_id, entrypoint_id).await
    }

    pub async fn handle_run_generated_entrypoint(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    ) {
        self.send_command(PluginCommand::One {
            id: plugin_id.clone(),
            data: OnePluginCommandData::RunGeneratedEntrypoint {
                entrypoint_id: entrypoint_id.to_string(),
                action_index,
            },
        });

        self.mark_entrypoint_frecency(plugin_id, entrypoint_id).await
    }

    pub async fn handle_render_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    ) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        self.send_command(PluginCommand::One {
            id: plugin_id.clone(),
            data: OnePluginCommandData::RenderView {
                entrypoint_id: entrypoint_id.clone(),
            },
        });

        self.mark_entrypoint_frecency(plugin_id.clone(), entrypoint_id.clone())
            .await;

        let shortcuts = self.action_shortcuts(plugin_id, entrypoint_id).await?;

        Ok(shortcuts)
    }

    pub fn handle_view_close(&self, plugin_id: PluginId) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::CloseView,
        })
    }

    pub fn handle_view_event(
        &self,
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    ) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::HandleViewEvent {
                widget_id,
                event_name,
                event_arguments,
            },
        })
    }

    pub fn handle_keyboard_event(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        origin: KeyboardEventOrigin,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::HandleKeyboardEvent {
                entrypoint_id,
                origin,
                key,
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            },
        })
    }

    pub fn request_search_index_refresh(&self, plugin_id: PluginId) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::RefreshSearchIndex,
        })
    }

    pub fn handle_open(&self, href: String) {
        match open::that_detached(&href) {
            Ok(()) => tracing::info!("Opened '{}' successfully.", href),
            Err(err) => tracing::error!("An error occurred when opening '{}': {}", href, err),
        }
    }

    pub fn handle_open_settings_window(&self) {
        let current_exe = std::env::current_exe().expect("unable to get current_exe");

        std::process::Command::new(current_exe)
            .args(["settings"])
            .spawn()
            .expect("failed to execute settings process");
    }

    pub fn handle_open_settings_window_preferences(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) {
        let data = if let Some(entrypoint_id) = entrypoint_id {
            SettingsEnvData::OpenEntrypointPreferences {
                plugin_id: plugin_id.to_string(),
                entrypoint_id: entrypoint_id.to_string(),
            }
        } else {
            SettingsEnvData::OpenPluginPreferences {
                plugin_id: plugin_id.to_string(),
            }
        };

        let current_exe = std::env::current_exe().expect("unable to get current_exe");

        std::process::Command::new(current_exe)
            .args(["settings"])
            .env(SETTINGS_ENV, settings_env_data_to_string(data))
            .spawn()
            .expect("failed to execute settings process"); // this can fail in dev if binary was replaced by more recent compilation
    }

    async fn reload_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Reloading plugin with id: {:?}", plugin_id);

        let running = self.run_status_holder.is_plugin_running(&plugin_id);
        if running {
            self.stop_plugin(plugin_id.clone()).await;
        }

        if self.is_plugin_enabled(&plugin_id).await? {
            self.start_plugin(plugin_id).await?;
        }

        Ok(())
    }

    async fn is_plugin_enabled(&self, plugin_id: &PluginId) -> anyhow::Result<bool> {
        self.db_repository.is_plugin_enabled(&plugin_id.to_string()).await
    }

    async fn action_shortcuts(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    ) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        self.db_repository
            .action_shortcuts(&plugin_id.to_string(), &entrypoint_id.to_string())
            .await
    }

    async fn start_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Starting plugin with id: {:?}", plugin_id);

        let plugin_id_str = plugin_id.to_string();

        let plugin = self.db_repository.get_plugin_by_id(&plugin_id_str).await?;

        let entrypoint_names = self
            .db_repository
            .get_entrypoints_by_plugin_id(&plugin_id_str)
            .await?
            .into_iter()
            .map(|entrypoint| (EntrypointId::from_string(entrypoint.id), entrypoint.name))
            .collect::<HashMap<EntrypointId, String>>();

        let inline_view_entrypoint_id = self
            .db_repository
            .get_inline_view_entrypoint_id_for_plugin(&plugin_id_str)
            .await?;

        let receiver = self.command_broadcaster.subscribe();

        let clipboard_permissions = plugin
            .permissions
            .clipboard
            .into_iter()
            .map(|permission| {
                match permission {
                    DbPluginClipboardPermissions::Read => PluginPermissionsClipboard::Read,
                    DbPluginClipboardPermissions::Write => PluginPermissionsClipboard::Write,
                    DbPluginClipboardPermissions::Clear => PluginPermissionsClipboard::Clear,
                }
            })
            .collect();

        let main_search_bar_permissions = plugin
            .permissions
            .main_search_bar
            .into_iter()
            .map(|permission| {
                match permission {
                    DbPluginMainSearchBarPermissions::Read => JsPluginPermissionsMainSearchBar::Read,
                }
            })
            .collect();

        let data = PluginRuntimeData {
            id: plugin_id,
            uuid: plugin.uuid,
            name: plugin.name,
            entrypoint_names,
            code: JsPluginCode { js: plugin.code.js },
            inline_view_entrypoint_id,
            permissions: PluginPermissions {
                environment: plugin.permissions.environment,
                network: plugin.permissions.network,
                filesystem: JsPluginPermissionsFileSystem {
                    read: plugin.permissions.filesystem.read,
                    write: plugin.permissions.filesystem.write,
                },
                exec: JsPluginPermissionsExec {
                    command: plugin.permissions.exec.command,
                    executable: plugin.permissions.exec.executable,
                },
                system: plugin.permissions.system,
                clipboard: clipboard_permissions,
                main_search_bar: main_search_bar_permissions,
            },
            command_receiver: receiver,
            db_repository: self.db_repository.clone(),
            search_index: self.search_index.clone(),
            icon_cache: self.icon_cache.clone(),
            frontend_api: self.frontend_api.clone(),
            dirs: self.dirs.clone(),
            clipboard: self.clipboard.clone(),
        };

        self.start_plugin_runtime(data);

        Ok(())
    }

    async fn stop_plugin(&self, plugin_id: PluginId) {
        tracing::info!(target = "plugin", "Stopping plugin with id: {:?}", plugin_id);

        self.run_status_holder.stop_plugin(&plugin_id)
    }

    fn start_plugin_runtime(&self, data: PluginRuntimeData) {
        let run_status_guard = self.run_status_holder.start_block(data.id.clone());

        tokio::spawn(async {
            start_plugin_runtime(data, run_status_guard)
                .await
                .expect("failed to start plugin runtime")
        });
    }

    fn send_command(&self, command: PluginCommand) {
        // it is possible to have 0 plugins
        let _ = self.command_broadcaster.send(command);
    }

    async fn mark_entrypoint_frecency(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) {
        let result = self
            .db_repository
            .mark_entrypoint_frecency(&plugin_id.to_string(), &entrypoint_id.to_string())
            .await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when marking entrypoint frecency {:?}",
                err
            )
        }

        self.request_search_index_refresh(plugin_id);
    }

    pub async fn inline_view_shortcuts(&self) -> anyhow::Result<HashMap<PluginId, HashMap<String, PhysicalShortcut>>> {
        let result: HashMap<_, _> = self
            .db_repository
            .inline_view_shortcuts()
            .await?
            .into_iter()
            .map(|(plugin_id, shortcuts)| (PluginId::from_string(plugin_id), shortcuts))
            .collect();

        Ok(result)
    }
}

fn plugin_preference_from_db(id: &str, value: DbPluginPreference) -> PluginPreference {
    match value {
        DbPluginPreference::Number {
            name,
            default,
            description,
        } => {
            PluginPreference::Number {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                description,
            }
        }
        DbPluginPreference::String {
            name,
            default,
            description,
        } => {
            PluginPreference::String {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                description,
            }
        }
        DbPluginPreference::Enum {
            name,
            default,
            description,
            enum_values,
        } => {
            let enum_values = enum_values
                .into_iter()
                .map(|value| {
                    PreferenceEnumValue {
                        label: value.label,
                        value: value.value,
                    }
                })
                .collect();

            PluginPreference::Enum {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                description,
                enum_values,
            }
        }
        DbPluginPreference::Bool {
            name,
            default,
            description,
        } => {
            PluginPreference::Bool {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                description,
            }
        }
        DbPluginPreference::ListOfStrings {
            name,
            default,
            description,
        } => {
            PluginPreference::ListOfStrings {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                description,
            }
        }
        DbPluginPreference::ListOfNumbers {
            name,
            default,
            description,
        } => {
            PluginPreference::ListOfNumbers {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                description,
            }
        }
        DbPluginPreference::ListOfEnums {
            name,
            default,
            enum_values,
            description,
        } => {
            let enum_values = enum_values
                .into_iter()
                .map(|value| {
                    PreferenceEnumValue {
                        label: value.label,
                        value: value.value,
                    }
                })
                .collect();

            PluginPreference::ListOfEnums {
                name: name.unwrap_or_else(|| id.to_string()),
                default,
                enum_values,
                description,
            }
        }
    }
}

fn plugin_preference_user_data_to_db(value: PluginPreferenceUserData) -> DbPluginPreferenceUserData {
    match value {
        PluginPreferenceUserData::Number { value } => DbPluginPreferenceUserData::Number { value },
        PluginPreferenceUserData::String { value } => DbPluginPreferenceUserData::String { value },
        PluginPreferenceUserData::Enum { value } => DbPluginPreferenceUserData::Enum { value },
        PluginPreferenceUserData::Bool { value } => DbPluginPreferenceUserData::Bool { value },
        PluginPreferenceUserData::ListOfStrings { value } => DbPluginPreferenceUserData::ListOfStrings { value },
        PluginPreferenceUserData::ListOfNumbers { value } => DbPluginPreferenceUserData::ListOfNumbers { value },
        PluginPreferenceUserData::ListOfEnums { value } => DbPluginPreferenceUserData::ListOfEnums { value },
    }
}

fn plugin_preference_user_data_from_db(value: DbPluginPreferenceUserData) -> PluginPreferenceUserData {
    match value {
        DbPluginPreferenceUserData::Number { value } => PluginPreferenceUserData::Number { value },
        DbPluginPreferenceUserData::String { value } => PluginPreferenceUserData::String { value },
        DbPluginPreferenceUserData::Enum { value } => PluginPreferenceUserData::Enum { value },
        DbPluginPreferenceUserData::Bool { value } => PluginPreferenceUserData::Bool { value },
        DbPluginPreferenceUserData::ListOfStrings { value, .. } => PluginPreferenceUserData::ListOfStrings { value },
        DbPluginPreferenceUserData::ListOfNumbers { value, .. } => PluginPreferenceUserData::ListOfNumbers { value },
        DbPluginPreferenceUserData::ListOfEnums { value, .. } => PluginPreferenceUserData::ListOfEnums { value },
    }
}
