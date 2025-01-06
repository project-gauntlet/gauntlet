use std::env::consts::OS;
use std::io::ErrorKind;
use std::path::PathBuf;
use anyhow::{anyhow, Context};
use dark_light::Mode;
use serde::{Deserialize, Serialize};
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::{UiTheme, UiThemeColor, UiThemeContent, UiThemeContentBorder, UiThemeMode, UiThemeWindow, UiThemeWindowBorder};
use gauntlet_common::rpc::frontend_api::FrontendApi;
use crate::plugins::data_db_repository::DataDbRepository;

pub struct BundledThemes {
    pub legacy_theme: UiTheme,
    pub macos_dark_theme: UiTheme,
    pub macos_light_theme: UiTheme,
}

const LEGACY_THEME: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../bundled_themes/legacy.toml"));
const MACOS_DARK_THEME: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../bundled_themes/macos_dark.toml"));
const MACOS_LIGHT_THEME: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../bundled_themes/macos_light.toml"));

impl BundledThemes {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            legacy_theme: parse_theme(LEGACY_THEME).expect("bundled theme should always be valid"),
            macos_dark_theme: parse_theme(MACOS_DARK_THEME).expect("bundled theme should always be valid"),
            macos_light_theme: parse_theme(MACOS_LIGHT_THEME).expect("bundled theme should always be valid"),
        })
    }
}

pub fn convert_theme(config_theme: ConfigTheme) -> anyhow::Result<UiTheme> {
    let [background_100, background_200, background_300, background_400] = config_theme.background;
    let [text_100, text_200, text_300, text_400] = config_theme.text;

    Ok(UiTheme {
        mode: match config_theme.mode {
            ConfigThemeMode::Light => UiThemeMode::Light,
            ConfigThemeMode::Dark => UiThemeMode::Dark
        },
        background: [
            convert_complex_color(background_100)?,
            convert_complex_color(background_200)?,
            convert_complex_color(background_300)?,
            convert_complex_color(background_400)?
        ],
        text: [
            convert_complex_color(text_100)?,
            convert_complex_color(text_200)?,
            convert_complex_color(text_300)?,
            convert_complex_color(text_400)?,
        ],
        window: UiThemeWindow {
            border: UiThemeWindowBorder {
                radius: config_theme.window.border.radius,
                width: config_theme.window.border.width,
                color: convert_complex_color(config_theme.window.border.color)?,
            },
        },
        content: UiThemeContent {
            border: UiThemeContentBorder {
                radius: config_theme.content.border.radius,
            },
        },
    })
}

fn convert_complex_color(color: ConfigThemeColor) -> anyhow::Result<UiThemeColor> {
    match color {
        ConfigThemeColor::String(value) => convert_color(value, true),
        ConfigThemeColor::Object { color, alpha } => {

            if !(0.0..=1.0).contains(&alpha) {
                Err(anyhow!("Alpha component must be on [0, 1] range"))?;
            }

            let mut color = convert_color(color, false)?;

            color.a = alpha;

            Ok(color)
        }
    }
}

fn convert_color(color: String, allow_alpha: bool) -> anyhow::Result<UiThemeColor> {
    if !color.starts_with("#") {
        Err(anyhow!("Colors have to start with #"))?;
    }

    let hex = color.strip_prefix('#').expect("validated just above");

    let parse_channel = |from: usize, to: usize| -> anyhow::Result<f32> {
        let num = usize::from_str_radix(&hex[from..=to], 16)
            .context("Unable to parse as hex number")?;
        let num = num as f32 / 255.0;

        // If we only got half a byte (one letter), expand it into a full byte (two letters)
        Ok(if from == to { num + num * 16.0 } else { num })
    };

    let color = match hex.len() {
        3 => UiThemeColor {
            r: parse_channel(0, 0)?,
            g: parse_channel(1, 1)?,
            b: parse_channel(2, 2)?,
            a: 1.0,
        },
        4 => {
            if allow_alpha {
                UiThemeColor {
                    r: parse_channel(0, 0)?,
                    g: parse_channel(1, 1)?,
                    b: parse_channel(2, 2)?,
                    a: parse_channel(3, 3)?,
                }
            } else {
                Err(anyhow!("alpha channel is not allowed here"))?
            }
        },
        6 => UiThemeColor {
            r: parse_channel(0, 1)?,
            g: parse_channel(2, 3)?,
            b: parse_channel(4, 5)?,
            a: 1.0,
        },
        8 => {
            if allow_alpha {
                UiThemeColor {
                    r: parse_channel(0, 1)?,
                    g: parse_channel(2, 3)?,
                    b: parse_channel(4, 5)?,
                    a: parse_channel(6, 7)?,
                }
            } else {
                Err(anyhow!("alpha channel is not allowed here"))?
            }
        },
        _ => Err(anyhow!("invalid length of a color string"))?,
    };

    Ok(color)
}

pub fn parse_theme(value: &str) -> anyhow::Result<UiTheme> {
    let value = toml::from_str::<ConfigTheme>(value)
        .context("Unable to parse theme file")?;

    match convert_theme(value) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.context("Unable to parse theme file"))
    }
}

pub fn read_theme_file(theme_file: PathBuf) -> Option<UiTheme> {
    match std::fs::read_to_string(&theme_file) {
        Ok(value) => {
            match parse_theme(&value) {
                Ok(value) => Some(value),
                Err(err) => {
                    tracing::warn!("Unable to parse theme file: {:?} - {:?}", theme_file, err);
                    None
                }
            }
        },
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    tracing::debug!("No theme file was found");
                    None
                },
                err @ _ => {
                    tracing::warn!("Unable to read theme file: {}", err);
                    None
                },
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigThemeMode {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigThemeColor {
    String(String),
    Object {
        color: String,
        alpha: f32
    }
}

pub type ConfigThemeColorPalette = [ConfigThemeColor; 4];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigThemeWindow {
    pub border: ConfigThemeWindowBorder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigThemeWindowBorder {
    pub radius: f32,
    pub width: f32,
    pub color: ConfigThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigThemeContent {
    pub border: ConfigThemeContentBorder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigThemeContentBorder {
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTheme {
    pub mode: ConfigThemeMode,
    // value of tint/tones/shades/whatever you have, from lower to higher
    pub background: ConfigThemeColorPalette,
    pub text: ConfigThemeColorPalette,
    pub window: ConfigThemeWindow,
    pub content: ConfigThemeContent,
}
