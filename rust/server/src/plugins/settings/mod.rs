pub mod global_shortcut;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use dark_light::Mode;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::SettingsTheme;
use gauntlet_common::model::UiTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::rpc::frontend_api::FrontendApi;
use gauntlet_common::rpc::frontend_api::FrontendApiProxy;
use global_hotkey::GlobalHotKeyManager;

use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::data_db_repository::DbSettingsEntrypointSearchAliasData;
use crate::plugins::data_db_repository::DbTheme;
use crate::plugins::data_db_repository::DbWindowPositionMode;
use crate::plugins::settings::global_shortcut::GlobalShortcutAction;
use crate::plugins::settings::global_shortcut::GlobalShortcutPressedEvent;
use crate::plugins::settings::global_shortcut::GlobalShortcutSettings;
use crate::plugins::theme::BundledThemes;
use crate::plugins::theme::read_theme_file;

#[derive(Clone)]
pub struct Settings {
    dirs: Dirs,
    repository: DataDbRepository,
    frontend_api: FrontendApiProxy,
    global_hotkey_settings: GlobalShortcutSettings,
    themes: Arc<BundledThemes>,
}

impl Settings {
    pub fn new(dirs: Dirs, repository: DataDbRepository, frontend_api: FrontendApiProxy) -> anyhow::Result<Self> {
        Ok(Self {
            dirs,
            repository: repository.clone(),
            frontend_api,
            global_hotkey_settings: GlobalShortcutSettings::new(repository)?,
            themes: Arc::new(BundledThemes::new()?),
        })
    }

    pub fn setup(&self, global_hotkey_manager: &GlobalHotKeyManager) -> anyhow::Result<()> {
        self.global_hotkey_settings.setup(global_hotkey_manager)?;

        Ok(())
    }

    pub fn handle_global_shortcut_event(
        &self,
        event: GlobalShortcutPressedEvent,
    ) -> anyhow::Result<GlobalShortcutAction> {
        self.global_hotkey_settings.handle_global_shortcut_event(event)
    }

    pub fn global_shortcut(&self) -> anyhow::Result<Option<(PhysicalShortcut, Option<String>)>> {
        self.global_hotkey_settings.global_shortcut()
    }

    pub fn set_global_shortcut(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        shortcut: Option<PhysicalShortcut>,
    ) -> anyhow::Result<()> {
        self.global_hotkey_settings
            .set_global_shortcut(global_hotkey_manager, shortcut)
    }

    pub fn global_entrypoint_shortcuts(
        &self,
    ) -> anyhow::Result<HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>> {
        self.global_hotkey_settings.global_entrypoint_shortcuts()
    }

    pub fn set_global_entrypoint_shortcut(
        &self,
        global_hotkey_manager: &GlobalHotKeyManager,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut: Option<PhysicalShortcut>,
    ) -> anyhow::Result<()> {
        self.global_hotkey_settings.set_global_entrypoint_shortcut(
            global_hotkey_manager,
            plugin_id,
            entrypoint_id,
            shortcut,
        )
    }

    pub fn entrypoint_search_aliases(&self) -> anyhow::Result<HashMap<(PluginId, EntrypointId), String>> {
        let settings = self.repository.get_settings()?;

        let data: HashMap<_, _> = settings
            .entrypoint_search_aliases
            .unwrap_or_default()
            .into_iter()
            .map(|data| {
                (
                    (
                        PluginId::from_string(data.plugin_id),
                        EntrypointId::from_string(data.entrypoint_id),
                    ),
                    data.alias,
                )
            })
            .collect();

        Ok(data)
    }

    pub fn set_entrypoint_search_alias(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        alias: Option<String>,
    ) -> anyhow::Result<()> {
        self.repository.mutate_settings(|mut settings| {
            let mut alias_data: HashMap<_, _> = settings
                .entrypoint_search_aliases
                .unwrap_or_default()
                .into_iter()
                .map(|data| {
                    (
                        (
                            PluginId::from_string(data.plugin_id),
                            EntrypointId::from_string(data.entrypoint_id),
                        ),
                        data.alias,
                    )
                })
                .collect();

            match alias {
                None => alias_data.remove(&(plugin_id, entrypoint_id)),
                Some(alias) => alias_data.insert((plugin_id, entrypoint_id), alias),
            };

            let alias_data = alias_data
                .into_iter()
                .map(|((plugin_id, entrypoint_id), alias)| {
                    DbSettingsEntrypointSearchAliasData {
                        plugin_id: plugin_id.to_string(),
                        entrypoint_id: entrypoint_id.to_string(),
                        alias,
                    }
                })
                .collect();

            settings.entrypoint_search_aliases = Some(alias_data);

            Ok(settings)
        })?;

        Ok(())
    }

    pub fn effective_theme(&self) -> anyhow::Result<UiTheme> {
        if let Some(theme) = read_theme_file(self.dirs.theme_file()) {
            return Ok(theme);
        };

        // TODO config

        let settings = self.repository.get_settings()?;

        let theme = match &settings.theme {
            None => self.autodetect_theme(),
            Some(theme) => {
                match theme {
                    DbTheme::MacOSLight => self.themes.macos_light_theme.clone(),
                    DbTheme::MacOSDark => self.themes.macos_dark_theme.clone(),
                    DbTheme::Legacy => self.themes.legacy_theme.clone(),
                }
            }
        };

        Ok(theme)
    }

    #[cfg(feature = "scenario_runner")]
    pub fn scenarios_theme(&self) -> UiTheme {
        self.themes.macos_dark_theme.clone()
    }

    pub fn theme_setting(&self) -> anyhow::Result<SettingsTheme> {
        if let Some(_) = read_theme_file(self.dirs.theme_file()) {
            return Ok(SettingsTheme::ThemeFile);
        };

        // TODO config

        let settings = self.repository.get_settings()?;

        match settings.theme {
            None => Ok(SettingsTheme::AutoDetect),
            Some(DbTheme::MacOSLight) => Ok(SettingsTheme::MacOSLight),
            Some(DbTheme::MacOSDark) => Ok(SettingsTheme::MacOSDark),
            Some(DbTheme::Legacy) => Ok(SettingsTheme::Legacy),
        }
    }

    pub async fn set_theme_setting(&self, theme: SettingsTheme) -> anyhow::Result<()> {
        let settings = self.repository.mutate_settings(|mut settings| {
            settings.theme = match &theme {
                SettingsTheme::AutoDetect => None,
                SettingsTheme::MacOSLight => Some(DbTheme::MacOSLight),
                SettingsTheme::MacOSDark => Some(DbTheme::MacOSDark),
                SettingsTheme::Legacy => Some(DbTheme::Legacy),
                // these should not be visible in settings ui
                SettingsTheme::Config => Err(anyhow!("Unable to set current theme to config"))?,
                SettingsTheme::ThemeFile => Err(anyhow!("Unable to set current theme to a file"))?,
            };

            Ok(settings)
        })?;

        let theme = match &settings.theme {
            None => self.autodetect_theme(),
            Some(theme) => {
                match theme {
                    DbTheme::MacOSLight => self.themes.macos_light_theme.clone(),
                    DbTheme::MacOSDark => self.themes.macos_dark_theme.clone(),
                    DbTheme::Legacy => self.themes.legacy_theme.clone(),
                }
            }
        };

        self.frontend_api.set_theme(theme).await?;

        Ok(())
    }

    pub fn window_position_mode_setting(&self) -> anyhow::Result<WindowPositionMode> {
        let settings = self.repository.get_settings()?;

        let window_position_mode = match &settings.window_position_mode {
            None => WindowPositionMode::Static,
            Some(DbWindowPositionMode::ActiveMonitor) => WindowPositionMode::ActiveMonitor,
        };

        Ok(window_position_mode)
    }

    pub async fn set_window_position_mode_setting(&self, mode: WindowPositionMode) -> anyhow::Result<()> {
        self.repository.mutate_settings(|mut settings| {
            let window_position_mode = match mode {
                WindowPositionMode::Static => None,
                WindowPositionMode::ActiveMonitor => Some(DbWindowPositionMode::ActiveMonitor),
            };

            settings.window_position_mode = window_position_mode;

            Ok(settings)
        })?;

        self.frontend_api.set_window_position_mode(mode).await?;

        Ok(())
    }

    fn autodetect_theme(&self) -> UiTheme {
        match dark_light::detect() {
            Mode::Dark => self.themes.macos_dark_theme.clone(),
            Mode::Light => self.themes.macos_light_theme.clone(),
            Mode::Default => self.themes.macos_dark_theme.clone(),
        }
    }
}
