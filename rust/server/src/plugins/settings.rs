use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use dark_light::Mode;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::SettingsTheme;
use gauntlet_common::model::UiTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::rpc::frontend_api::FrontendApi;
use gauntlet_common::rpc::frontend_api::FrontendApiProxy;

use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::data_db_repository::DbSettingsEntrypointSearchAliasData;
use crate::plugins::data_db_repository::DbSettingsGlobalEntrypointShortcutData;
use crate::plugins::data_db_repository::DbSettingsGlobalShortcutData;
use crate::plugins::data_db_repository::DbSettingsShortcut;
use crate::plugins::data_db_repository::DbTheme;
use crate::plugins::data_db_repository::DbWindowPositionMode;
use crate::plugins::theme::BundledThemes;
use crate::plugins::theme::read_theme_file;

#[derive(Clone)]
pub struct Settings {
    dirs: Dirs,
    repository: DataDbRepository,
    frontend_api: FrontendApiProxy,
    themes: Arc<BundledThemes>,
}

impl Settings {
    pub fn new(dirs: Dirs, repository: DataDbRepository, frontend_api: FrontendApiProxy) -> anyhow::Result<Self> {
        Ok(Self {
            dirs,
            repository,
            frontend_api,
            themes: Arc::new(BundledThemes::new()?),
        })
    }

    pub async fn global_shortcut(&self) -> anyhow::Result<Option<(PhysicalShortcut, Option<String>)>> {
        let settings = self.repository.get_settings().await?;

        let data = settings.global_shortcut.map(|data| {
            let shortcut = PhysicalShortcut {
                physical_key: PhysicalKey::from_value(data.shortcut.physical_key),
                modifier_shift: data.shortcut.modifier_shift,
                modifier_control: data.shortcut.modifier_control,
                modifier_alt: data.shortcut.modifier_alt,
                modifier_meta: data.shortcut.modifier_meta,
            };

            (shortcut, data.error)
        });

        Ok(data)
    }

    pub async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> anyhow::Result<()> {
        let err = self.frontend_api.set_global_shortcut(shortcut.clone()).await;

        let db_err = err.as_ref().map_err(|err| format!("{:#}", err)).err();

        let mut settings = self.repository.get_settings().await?;

        settings.global_shortcut = shortcut.map(|shortcut| {
            DbSettingsGlobalShortcutData {
                shortcut: DbSettingsShortcut {
                    physical_key: shortcut.physical_key.to_value(),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                },
                error: db_err,
            }
        });

        self.repository.set_settings(settings).await?;

        err.map_err(Into::into)
    }

    pub async fn set_global_shortcut_error(&self, error: Option<String>) -> anyhow::Result<()> {
        let mut settings = self.repository.get_settings().await?;

        if let Some(data) = &mut settings.global_shortcut {
            data.error = error
        }

        self.repository.set_settings(settings).await?;

        Ok(())
    }

    pub async fn global_entrypoint_shortcuts(
        &self,
    ) -> anyhow::Result<HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>> {
        let settings = self.repository.get_settings().await?;
        let data: HashMap<_, _> = settings
            .global_entrypoint_shortcuts
            .unwrap_or_default()
            .into_iter()
            .map(|data| {
                let shortcut = data.shortcut.shortcut;
                let error = data.shortcut.error;

                let shortcut = PhysicalShortcut {
                    physical_key: PhysicalKey::from_value(shortcut.physical_key),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                };

                (
                    (
                        PluginId::from_string(data.plugin_id),
                        EntrypointId::from_string(data.entrypoint_id),
                    ),
                    (shortcut, error),
                )
            })
            .collect();

        Ok(data)
    }

    pub async fn set_global_entrypoint_shortcut(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut: Option<PhysicalShortcut>,
    ) -> anyhow::Result<()> {
        let err = self
            .frontend_api
            .set_global_entrypoint_shortcut(plugin_id.clone(), entrypoint_id.clone(), shortcut.clone())
            .await;

        let db_err = err.as_ref().map_err(|err| format!("{:#}", err)).err();

        let mut settings = self.repository.get_settings().await?;

        let mut shortcuts: HashMap<_, _> = settings
            .global_entrypoint_shortcuts
            .unwrap_or_default()
            .into_iter()
            .map(|data| {
                (
                    (
                        PluginId::from_string(data.plugin_id),
                        EntrypointId::from_string(data.entrypoint_id),
                    ),
                    data.shortcut,
                )
            })
            .collect();

        match shortcut {
            None => {
                shortcuts.remove(&(plugin_id, entrypoint_id));
            }
            Some(shortcut) => {
                shortcuts.insert(
                    (plugin_id, entrypoint_id),
                    DbSettingsGlobalShortcutData {
                        shortcut: DbSettingsShortcut {
                            physical_key: shortcut.physical_key.to_value(),
                            modifier_shift: shortcut.modifier_shift,
                            modifier_control: shortcut.modifier_control,
                            modifier_alt: shortcut.modifier_alt,
                            modifier_meta: shortcut.modifier_meta,
                        },
                        error: db_err,
                    },
                );
            }
        }

        let global_entrypoint_shortcuts = shortcuts
            .into_iter()
            .map(|((plugin_id, entrypoint_id), data)| {
                DbSettingsGlobalEntrypointShortcutData {
                    plugin_id: plugin_id.to_string(),
                    entrypoint_id: entrypoint_id.to_string(),
                    shortcut: data,
                }
            })
            .collect();

        settings.global_entrypoint_shortcuts = Some(global_entrypoint_shortcuts);

        self.repository.set_settings(settings).await?;

        Ok(())
    }

    pub async fn set_global_entrypoint_shortcut_error(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        error: Option<String>,
    ) -> anyhow::Result<()> {
        let mut settings = self.repository.get_settings().await?;

        let mut shortcuts: HashMap<_, _> = settings
            .global_entrypoint_shortcuts
            .unwrap_or_default()
            .into_iter()
            .map(|data| {
                (
                    (
                        PluginId::from_string(data.plugin_id),
                        EntrypointId::from_string(data.entrypoint_id),
                    ),
                    data.shortcut,
                )
            })
            .collect();

        if let Some(data) = shortcuts.get_mut(&(plugin_id, entrypoint_id)) {
            data.error = error;
        };

        let global_entrypoint_shortcuts = shortcuts
            .into_iter()
            .map(|((plugin_id, entrypoint_id), data)| {
                DbSettingsGlobalEntrypointShortcutData {
                    plugin_id: plugin_id.to_string(),
                    entrypoint_id: entrypoint_id.to_string(),
                    shortcut: data,
                }
            })
            .collect();

        settings.global_entrypoint_shortcuts = Some(global_entrypoint_shortcuts);

        self.repository.set_settings(settings).await?;

        Ok(())
    }

    pub async fn entrypoint_search_aliases(&self) -> anyhow::Result<HashMap<(PluginId, EntrypointId), String>> {
        let settings = self.repository.get_settings().await?;

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

    pub async fn set_entrypoint_search_alias(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        alias: Option<String>,
    ) -> anyhow::Result<()> {
        let mut settings = self.repository.get_settings().await?;

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

        self.repository.set_settings(settings).await?;

        Ok(())
    }

    pub async fn effective_theme(&self) -> anyhow::Result<UiTheme> {
        if let Some(theme) = read_theme_file(self.dirs.theme_file()) {
            return Ok(theme);
        };

        // TODO config

        let settings = self.repository.get_settings().await?;

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

    pub async fn theme_setting(&self) -> anyhow::Result<SettingsTheme> {
        if let Some(_) = read_theme_file(self.dirs.theme_file()) {
            return Ok(SettingsTheme::ThemeFile);
        };

        // TODO config

        let settings = self.repository.get_settings().await?;

        match settings.theme {
            None => Ok(SettingsTheme::AutoDetect),
            Some(DbTheme::MacOSLight) => Ok(SettingsTheme::MacOSLight),
            Some(DbTheme::MacOSDark) => Ok(SettingsTheme::MacOSDark),
            Some(DbTheme::Legacy) => Ok(SettingsTheme::Legacy),
        }
    }

    pub async fn set_theme_setting(&self, theme: SettingsTheme) -> anyhow::Result<()> {
        let mut settings = self.repository.get_settings().await?;

        settings.theme = match theme {
            SettingsTheme::AutoDetect => None,
            SettingsTheme::MacOSLight => Some(DbTheme::MacOSLight),
            SettingsTheme::MacOSDark => Some(DbTheme::MacOSDark),
            SettingsTheme::Legacy => Some(DbTheme::Legacy),
            // these should not be visible in settings ui
            SettingsTheme::Config => Err(anyhow!("Unable to set current theme to config"))?,
            SettingsTheme::ThemeFile => Err(anyhow!("Unable to set current theme to a file"))?,
        };

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

        self.repository.set_settings(settings).await?;

        self.frontend_api.set_theme(theme).await?;

        Ok(())
    }

    pub async fn window_position_mode_setting(&self) -> anyhow::Result<WindowPositionMode> {
        let settings = self.repository.get_settings().await?;

        let window_position_mode = match &settings.window_position_mode {
            None => WindowPositionMode::Static,
            Some(DbWindowPositionMode::ActiveMonitor) => WindowPositionMode::ActiveMonitor,
        };

        Ok(window_position_mode)
    }

    pub async fn set_window_position_mode_setting(&self, mode: WindowPositionMode) -> anyhow::Result<()> {
        let mut settings = self.repository.get_settings().await?;

        let window_position_mode = match mode {
            WindowPositionMode::Static => None,
            WindowPositionMode::ActiveMonitor => Some(DbWindowPositionMode::ActiveMonitor),
        };

        settings.window_position_mode = window_position_mode;

        self.repository.set_settings(settings).await?;

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
