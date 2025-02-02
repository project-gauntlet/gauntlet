use std::env::consts::OS;

use anyhow::anyhow;
use dark_light::Mode;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::SettingsTheme;
use gauntlet_common::model::UiTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::rpc::frontend_api::FrontendApi;

use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::data_db_repository::DbTheme;
use crate::plugins::data_db_repository::DbWindowPositionMode;
use crate::plugins::theme::read_theme_file;
use crate::plugins::theme::BundledThemes;

pub struct Settings {
    dirs: Dirs,
    repository: DataDbRepository,
    frontend_api: FrontendApi,
    themes: BundledThemes,
}

impl Settings {
    pub fn new(dirs: Dirs, repository: DataDbRepository, frontend_api: FrontendApi) -> anyhow::Result<Self> {
        Ok(Self {
            dirs,
            repository,
            frontend_api,
            themes: BundledThemes::new()?,
        })
    }

    pub async fn effective_global_shortcut(&self) -> anyhow::Result<Option<PhysicalShortcut>> {
        match self.global_shortcut().await? {
            None => {
                if cfg!(target_os = "windows") {
                    Ok(Some(PhysicalShortcut {
                        physical_key: PhysicalKey::Space,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: true,
                        modifier_meta: false,
                    }))
                } else {
                    Ok(Some(PhysicalShortcut {
                        physical_key: PhysicalKey::Space,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: true,
                    }))
                }
            }
            Some((shortcut, _)) => Ok(shortcut),
        }
    }

    pub async fn global_shortcut(&self) -> anyhow::Result<Option<(Option<PhysicalShortcut>, Option<String>)>> {
        self.repository.get_global_shortcut().await
    }

    pub async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> anyhow::Result<()> {
        let err = self.frontend_api.set_global_shortcut(shortcut.clone()).await;

        let db_err = err.as_ref().map_err(|err| format!("{:#}", err)).err();

        self.repository.set_global_shortcut(shortcut, db_err).await?;

        err
    }

    pub async fn set_global_shortcut_error(&self, error: Option<String>) -> anyhow::Result<()> {
        match self.repository.get_global_shortcut().await? {
            None => {}
            Some((shortcut, _)) => {
                self.repository.set_global_shortcut(shortcut, error).await?;
            }
        };

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

        let mut settings = self.repository.get_settings().await?;

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
        let mut settings = self.repository.get_settings().await?;

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
