use std::io::ErrorKind;
use std::path::PathBuf;
use iced::{application, Color, Padding};
use iced::application::DefaultStyle;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use gauntlet_common::dirs::Dirs;

pub mod button;
pub mod text_input;
pub mod row;
pub mod container;
pub mod text;
pub mod date_picker;
pub mod image;
pub mod pick_list;
pub mod checkbox;
pub mod scrollable;
pub mod rule;
pub mod space;
pub mod grid;
pub mod tooltip;
mod loading_bar;

pub type Element<'a, Message> = iced::Element<'a, Message, GauntletComplexTheme>;

const CURRENT_SIMPLE_THEME_VERSION: u64 = 5;
const CURRENT_COMPLEX_THEME_VERSION: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GauntletSimpleThemeMode {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark
}

pub type GauntletSimpleThemeColorPalette = [ThemeColor; 4];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletSimpleThemeWindow {
    border: GauntletSimpleThemeWindowBorder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletSimpleThemeWindowBorder {
    radius: f32,
    width: f32,
    color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletSimpleThemeContent {
    border: GauntletSimpleThemeContentBorder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletSimpleThemeContentBorder {
    radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletSimpleTheme {
    version: u64,
    mode: GauntletSimpleThemeMode,
    // value of tint/tones/shades/whatever you have, from lower to higher
    background: GauntletSimpleThemeColorPalette,
    text: GauntletSimpleThemeColorPalette,
    window: GauntletSimpleThemeWindow,
    content: GauntletSimpleThemeContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletComplexTheme {
    version: u64,
    text: ThemeColor,
    root: ThemeRoot,
    popup: ThemeRoot,
    action: ThemeButton,
    action_panel: ThemePaddingBackgroundColor,
    action_panel_title: ThemePaddingOnly,
    action_shortcut: ThemePaddingOnly,
    action_shortcut_modifier: ThemeActionShortcutModifier,
    content_code_block: ThemePaddingOnly,
    content_code_block_text: ThemeCode,
    content_horizontal_break: ThemePaddingOnly,
    content_image: ThemeImage,
    content_paragraph: ThemePaddingOnly,
    detail_content: ThemePaddingOnly,
    detail_metadata: ThemePaddingOnly,
    empty_view_image: ThemePaddingSize,
    empty_view_subtitle: ThemeTextColor,
    form: ThemePaddingOnly,
    form_inner: ThemePaddingOnly,
    form_input: ThemePaddingOnly,
    form_input_label: ThemePaddingOnly,
    form_input_date_picker: ThemeDatePicker,
    form_input_date_picker_buttons: ThemeButton,
    form_input_checkbox: ThemeCheckbox,
    form_input_select: ThemeSelect,
    form_input_select_menu: ThemeSelectMenu,
    form_input_text_field: ThemeTextField,
    grid: ExternalThemeGrid,
    grid_inner: ThemePaddingOnly,
    list: ThemePaddingOnly,
    list_inner: ThemePaddingOnly,
    grid_item: ThemeButton,
    grid_item_title: ThemePaddingTextColor,
    grid_item_subtitle: ThemeTextColor,
    grid_section_title: ThemePaddingTextColorSpacing,
    grid_section_subtitle: ThemeTextColor,
    inline: ThemePaddingOnly,
    inline_inner: ThemeInline,
    inline_name: ThemePaddingTextColor,
    inline_separator: ThemeTextColor,
    list_item: ThemeButton,
    list_item_subtitle: ThemePaddingTextColor,
    list_item_title: ThemePaddingOnly,
    list_item_icon: ThemePaddingOnly,
    list_section_title: ThemePaddingTextColorSpacing,
    list_section_subtitle: ThemeTextColor,
    main_list: ThemePaddingOnly,
    main_list_inner: ThemePaddingOnly,
    main_list_item: ThemeButton,
    main_list_item_icon: ThemePaddingOnly,
    main_list_item_sub_text: ThemePaddingTextColor,
    main_list_item_text: ThemePaddingOnly,
    main_search_bar: ThemePaddingOnly,
    metadata_item_value: ThemePaddingOnly,
    metadata_content_inner: ThemePaddingOnly,
    metadata_inner: ThemePaddingOnly,
    metadata_separator: ThemePaddingOnly,
    metadata_tag_item: ThemePaddingOnly,
    metadata_item_label: ThemePaddingTextColorSize,
    metadata_link_icon: ThemePaddingOnly,
    metadata_tag_item_button: ThemeButton,
    plugin_error_view_description: ThemePaddingOnly,
    plugin_error_view_title: ThemePaddingOnly,
    preference_required_view_description: ThemePaddingOnly,
    root_bottom_panel: ThemeBottomPanel,
    root_bottom_panel_action_toggle_button: ThemeButton,
    root_bottom_panel_action_toggle_text: ThemePaddingTextColor,
    root_bottom_panel_primary_action_text: ThemePaddingTextColor,
    root_top_panel: ThemePaddingSpacing,
    root_top_panel_button: ThemeButton,
    metadata_link: ThemeLink,
    separator: ThemeSeparator,
    scrollbar: ThemeScrollbar,
    tooltip: ThemeTooltip,
    loading_bar: ThemeLoadingBar,
    text_accessory: ThemePaddingTextColorSpacing,
    icon_accessory: ThemeIconAccessory,
    hud: ThemeRoot,
    hud_content: ThemePaddingOnly
}

impl Default for GauntletComplexTheme {
    fn default() -> Self {
        panic!("should not be called")
    }
}

fn parse_json_theme<T: Serialize + DeserializeOwned>(theme_file: PathBuf, theme_name: &str) -> Option<T> {
    match std::fs::read_to_string(theme_file) {
        Ok(value) => {
            let result = serde_json::from_str::<serde_json::Value>(&value);

            match result {
                Ok(value) => {
                    match value.get("version") {
                        Some(serde_json::Value::Number(number)) => {
                            match number.as_u64() {
                                None => {
                                    tracing::warn!("Version of read {} file is invalid", theme_name);
                                    None
                                }
                                Some(CURRENT_SIMPLE_THEME_VERSION) => {
                                    match serde_json::from_value::<T>(value) {
                                        Ok(value) => Some(value),
                                        Err(err) => {
                                            tracing::warn!("Unable to parse {} file: {}", theme_name, err);
                                            None
                                        }
                                    }
                                }
                                Some(_) => {
                                    tracing::warn!("Version of read {} file doesn't match expected, theme: {}, expected: {}", theme_name, number, CURRENT_SIMPLE_THEME_VERSION);
                                    None
                                }
                            }
                        }
                        _ => {
                            tracing::warn!("Version of read {} file is not a number", theme_name);
                            None
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("Unable to parse {} file: {}", theme_name, err);
                    None
                }
            }
        }
        Err(err) => {
            match err.kind() {
                ErrorKind::NotFound => {
                    tracing::debug!("No {} file was found", theme_name);
                    None
                }
                err @ _ => {
                    tracing::warn!("Unable to read {} file: {}", theme_name, err);
                    None
                }
            }
        }
    }
}

// TODO padding on button is padding, not margin, a lot of margins missing?

impl GauntletComplexTheme {
    pub fn new() -> Self {
        let dirs = Dirs::new();

        let theme = parse_json_theme(dirs.complex_theme_file(), "complex theme")
            .unwrap_or_else(|| {
                let simple_theme = parse_json_theme(dirs.theme_simple_file(), "simple theme")
                    .unwrap_or_else(|| GauntletComplexTheme::default_simple_theme());

                GauntletComplexTheme::default_theme(simple_theme)
            });

        init_theme(theme.clone());

        theme
    }

    // TODO legacy
    pub fn default_simple_theme() -> GauntletSimpleTheme {
        GauntletSimpleTheme {
            version: CURRENT_SIMPLE_THEME_VERSION,
            mode: GauntletSimpleThemeMode::Dark,
            background: [
                ThemeColor::new(0x626974, 0.3),
                ThemeColor::new(0x48505B, 0.5),
                ThemeColor::new(0x333a42, 1.0),
                ThemeColor::new(0x2C323A, 1.0)
            ],
            text: [
                ThemeColor::new(0xDDDFE1, 1.0),
                ThemeColor::new(0x9AA0A6, 1.0),
                ThemeColor::new(0x6B7785, 1.0),
                ThemeColor::new(0x1D242C, 1.0),
            ],
            window: GauntletSimpleThemeWindow {
                border: GauntletSimpleThemeWindowBorder {
                    radius: 10.0,
                    width: 1.0,
                    color: ThemeColor::new(0x48505B, 0.5)
                },
            },
            content: GauntletSimpleThemeContent {
                border: GauntletSimpleThemeContentBorder {
                    radius: 4.0,
                },
            },
        }
    }

    // TODO auto detect macos dark mode
    // pub fn default_simple_theme() -> GauntletSimpleTheme {
    //     GauntletSimpleTheme {
    //         version: CURRENT_SIMPLE_THEME_VERSION,
    //         mode: GauntletSimpleThemeMode::Dark,
    //         background: [
    //             ThemeColor::from_rgba8(100, 100, 100, 0.5),
    //             ThemeColor::from_rgba8(55, 55, 55, 1.0),
    //             ThemeColor::from_rgba8(45, 45, 45, 1.0),
    //             ThemeColor::from_rgba8(36, 36, 36, 1.0),
    //         ],
    //         text: [
    //             ThemeColor::from_rgba8(229,229,229, 1.0),
    //             ThemeColor::from_rgba8(200, 200, 200, 1.0),
    //             ThemeColor::from_rgba8(150, 150, 150, 1.0),
    //             ThemeColor::from_rgba8(50, 50, 50, 1.0),
    //         ],
    //         window: GauntletSimpleThemeWindow {
    //             border: GauntletSimpleThemeWindowBorder {
    //                 radius:  8.0,
    //                 width: 1.0,
    //                 color: ThemeColor::from_rgba8(100, 100, 100, 1.0),
    //             },
    //         },
    //         content: GauntletSimpleThemeContent {
    //             border: GauntletSimpleThemeContentBorder {
    //                 radius: 4.0,
    //             },
    //         },
    //     }
    // }

    // TODO auto detect macos light mode
    // pub fn default_simple_theme() -> GauntletSimpleTheme {
    //     GauntletSimpleTheme {
    //         version: CURRENT_SIMPLE_THEME_VERSION,
    //         mode: GauntletSimpleThemeMode::Light,
    //         background: [
    //             ThemeColor::from_rgba8(0, 0, 0, 0.2),
    //             ThemeColor::from_rgba8(200, 200, 200, 1.0),
    //             ThemeColor::from_rgba8(210, 210, 210, 1.0),
    //             ThemeColor::from_rgba8(234, 234, 234, 1.0),
    //         ],
    //         text: [
    //             ThemeColor::from_rgba8(0, 0, 0, 1.0),
    //             ThemeColor::from_rgba8(0, 0, 0, 0.847),
    //             ThemeColor::from_rgba8(0, 0, 0, 0.498),
    //             ThemeColor::from_rgba8(0, 0, 0, 0.259),
    //         ],
    //         window: GauntletSimpleThemeWindow {
    //             border: GauntletSimpleThemeWindowBorder {
    //                 radius: 8.0,
    //                 width: 1.0,
    //                 color: ThemeColor::from_rgba8(185, 185, 185, 1.0),
    //             },
    //         },
    //         content: GauntletSimpleThemeContent {
    //             border: GauntletSimpleThemeContentBorder {
    //                 radius: 4.0,
    //             },
    //         },
    //     }
    // }

    pub fn default_theme(simple_theme: GauntletSimpleTheme) -> GauntletComplexTheme {
        let GauntletSimpleTheme {
            version: _,
            mode,
            background,
            text,
            window,
            content
        } = simple_theme;

        let [background_100, background_200, background_300, background_400] = background;
        let [text_100, text_200, text_300, _text_400] = text;

        GauntletComplexTheme {
            version: CURRENT_COMPLEX_THEME_VERSION,
            text: text_100,
            root: ThemeRoot {
                background_color: background_400,
                #[cfg(not(target_os = "macos"))]
                border_radius: window.border.radius,
                #[cfg(not(target_os = "macos"))]
                border_width: window.border.width,
                #[cfg(not(target_os = "macos"))]
                border_color: window.border.color,
                #[cfg(target_os = "macos")]
                border_radius: 0.0,
                #[cfg(target_os = "macos")]
                border_width: 0.0,
                #[cfg(target_os = "macos")]
                border_color: TRANSPARENT,
            },
            popup: ThemeRoot {
                background_color: background_400,
                border_radius: window.border.radius,
                border_width: window.border.width,
                border_color: window.border.color,
            },
            action_panel: ThemePaddingBackgroundColor {
                padding: padding_all(8.0),
                background_color: background_400,
            },
            action_panel_title: ThemePaddingOnly {
                padding: padding(2.0, 8.0, 4.0, 8.0),
            },
            action: ThemeButton {
                padding: padding_all(8.0),
                background_color: TRANSPARENT,
                background_color_focused: background_100,
                background_color_hovered: background_300,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            action_shortcut: ThemePaddingOnly {
                padding: padding_all(0.0)
            },
            action_shortcut_modifier: ThemeActionShortcutModifier {
                padding: padding_axis(0.0, 8.0),
                spacing: 8.0,
                background_color: background_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            form_input: ThemePaddingOnly {
                padding: padding_all(8.0)
            },
            metadata_tag_item: ThemePaddingOnly {
                padding: padding(0.0, 8.0, 4.0, 0.0),
            },
            metadata_tag_item_button: ThemeButton {
                padding: padding_axis(2.0, 8.0),
                background_color: match mode {
                    GauntletSimpleThemeMode::Light => background_300,
                    GauntletSimpleThemeMode::Dark => background_200
                },
                background_color_focused: match mode {
                    GauntletSimpleThemeMode::Light => background_200,
                    GauntletSimpleThemeMode::Dark => background_100
                },
                background_color_hovered: match mode {
                    GauntletSimpleThemeMode::Light => background_200,
                    GauntletSimpleThemeMode::Dark => background_100
                },
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            metadata_item_label: ThemePaddingTextColorSize {
                padding: padding_all(0.0),
                text_color: text_300,
                text_size: 14.0,
            },
            metadata_item_value: ThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            metadata_link_icon: ThemePaddingOnly {
                padding: padding_axis(0.0, 4.0),
            },
            list_item_subtitle: ThemePaddingTextColor {
                padding: padding_all(4.0),
                text_color: text_200,
            },
            list_item_title: ThemePaddingOnly {
                padding: padding_all(4.0),
            },
            content_paragraph: ThemePaddingOnly {
                padding: padding_all(8.0)
            },
            content_code_block: ThemePaddingOnly {
                padding: padding_all(0.0),
            },
            content_image: ThemeImage {
                padding: padding_all(0.0),
                border_radius: content.border.radius,
            },
            inline: ThemePaddingOnly {
                padding: padding_axis(0.0, 8.0),
            },
            inline_name: ThemePaddingTextColor {
                padding: padding_all(8.0),
                text_color: text_200,
            },
            inline_separator: ThemeTextColor {
                text_color: text_200,
            },
            inline_inner: ThemeInline {
                padding: padding_all(8.0),
                background_color: background_200,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            empty_view_image: ThemePaddingSize {
                padding: padding_all(8.0),
                size: ExternalThemeSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            grid_item: ThemeButton {
                padding: padding_all(8.0),
                background_color: background_200,
                background_color_focused: background_300,
                background_color_hovered: background_100,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            grid_item_title: ThemePaddingTextColor {
                padding: padding_axis(4.0, 0.0),
                text_color: text_100,
            },
            grid_item_subtitle: ThemeTextColor {
                text_color: text_200,
            },
            content_horizontal_break: ThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            content_code_block_text: ThemeCode {
                padding: padding_axis(4.0, 8.0),
                background_color: background_200,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            metadata_separator: ThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            root_top_panel: ThemePaddingSpacing {
                padding: padding_all(12.0),
                spacing: 12.0,
            },
            root_top_panel_button: ThemeButton {
                padding: padding_axis(3.0, 5.0),
                background_color: match mode {
                    GauntletSimpleThemeMode::Light => background_300,
                    GauntletSimpleThemeMode::Dark => background_200
                },
                background_color_focused: match mode {
                    GauntletSimpleThemeMode::Light => background_200,
                    GauntletSimpleThemeMode::Dark => background_100
                },
                background_color_hovered: match mode {
                    GauntletSimpleThemeMode::Light => background_200,
                    GauntletSimpleThemeMode::Dark => background_100
                },
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            root_bottom_panel: ThemeBottomPanel {
                padding: padding_axis(6.0, 8.0),
                background_color: background_300,
                spacing: 8.0
            },
            root_bottom_panel_action_toggle_button: ThemeButton {
                padding: padding_axis(3.0, 5.0),
                background_color: TRANSPARENT,
                background_color_focused: background_200,
                background_color_hovered: background_200,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            root_bottom_panel_action_toggle_text: ThemePaddingTextColor {
                padding: padding(0.0, 8.0, 0.0, 4.0),
                text_color: text_200
            },
            root_bottom_panel_primary_action_text: ThemePaddingTextColor {
                padding: padding(0.0, 8.0, 0.0, 4.0),
                text_color: text_100
            },
            list_item: ThemeButton {
                padding: padding_all(5.0),
                background_color: TRANSPARENT,
                background_color_focused: background_200,
                background_color_hovered: background_300,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            list_item_icon: ThemePaddingOnly {
                padding: padding_axis(0.0, 4.0)
            },
            detail_metadata: ThemePaddingOnly {
                padding: padding_axis(0.0, 12.0),
            },
            metadata_inner: ThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            detail_content: ThemePaddingOnly {
                padding: padding_axis(0.0, 12.0),
            },
            metadata_content_inner: ThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            form: ThemePaddingOnly {
                padding: padding_axis(0.0, 12.0),
            },
            form_inner: ThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            grid: ExternalThemeGrid {
                spacing: 8.0,
                padding: padding_axis(0.0, 12.0),
            },
            grid_inner: ThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            list: ThemePaddingOnly {
                padding: padding_axis(0.0, 8.0),
            },
            list_inner: ThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            form_input_label: ThemePaddingOnly {
                padding: padding_axis(4.0, 12.0),
            },
            list_section_title: ThemePaddingTextColorSpacing {
                padding: padding(12.0, 8.0, 4.0, 8.0),
                text_color: text_200,
                spacing: 8.0,
            },
            list_section_subtitle: ThemeTextColor {
                text_color: text_300
            },
            grid_section_title: ThemePaddingTextColorSpacing {
                padding: padding(12.0, 0.0, 4.0, 0.0),
                text_color: text_200,
                spacing: 8.0,
            },
            grid_section_subtitle: ThemeTextColor {
                text_color: text_300
            },
            main_list_item: ThemeButton {
                padding: padding_all(5.0),
                background_color: TRANSPARENT,
                background_color_focused: match mode {
                    GauntletSimpleThemeMode::Light => background_300,
                    GauntletSimpleThemeMode::Dark => background_200
                },
                background_color_hovered: match mode {
                    GauntletSimpleThemeMode::Light => background_200,
                    GauntletSimpleThemeMode::Dark => background_300
                },
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            main_list_item_text: ThemePaddingOnly {
                padding: padding_all(4.0),
            },
            main_list_item_sub_text: ThemePaddingTextColor {
                padding: padding_axis(4.0, 12.0),
                text_color: text_300,
            },
            main_list_item_icon: ThemePaddingOnly {
                padding: padding(0.0, 7.0, 0.0, 5.0),
            },
            main_list: ThemePaddingOnly {
                padding: padding_axis(0.0, 8.0),
            },
            main_list_inner: ThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            main_search_bar: ThemePaddingOnly {
                padding: padding_all(12.0),
            },
            plugin_error_view_title: ThemePaddingOnly {
                padding: padding_all(12.0),
            },
            plugin_error_view_description: ThemePaddingOnly {
                padding: padding_all(12.0),
            },
            preference_required_view_description: ThemePaddingOnly {
                padding: padding_all(12.0),
            },
            metadata_link: ThemeLink {
                text_color: text_100,
                text_color_hovered: text_200,
            },
            empty_view_subtitle: ThemeTextColor {
                text_color: text_300,
            },
            form_input_date_picker: ThemeDatePicker {
                background_color: background_400,
                text_color: text_100,
                text_color_selected: text_300,
                text_color_hovered: text_300,
                text_attenuated_color: text_300,
                day_background_color: background_200,
                day_background_color_selected: background_200,
                day_background_color_hovered: background_200,
            },
            form_input_date_picker_buttons: ThemeButton {
                padding: padding_all(8.0),
                background_color: background_200,
                background_color_focused: background_100,
                background_color_hovered: background_100,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            form_input_checkbox: ThemeCheckbox {
                background_color_checked: text_200,
                background_color_unchecked: TRANSPARENT,
                background_color_checked_hovered: text_100,
                background_color_unchecked_hovered: background_200,
                border_radius: content.border.radius,
                border_width: window.border.width,
                border_color: background_200,
                icon_color: background_400,
            },
            form_input_select: ThemeSelect {
                background_color: background_200,
                background_color_hovered: background_100,
                text_color: text_200,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: window.border.width,
                border_color: background_200,
            },
            form_input_select_menu: ThemeSelectMenu {
                background_color: background_400,
                background_color_selected: background_200,
                text_color: text_100,
                text_color_selected: text_100,
                border_radius: content.border.radius,
                border_width: window.border.width,
                border_color: background_200,
            },
            form_input_text_field: ThemeTextField {
                background_color: TRANSPARENT,
                background_color_hovered: background_200,
                text_color: text_100,
                text_color_placeholder: text_300,
                selection_color: background_200,
                border_radius: content.border.radius,
                border_width: window.border.width,
                border_color: background_200,
                border_color_hovered: background_200,
            },
            separator: ThemeSeparator {
                color: background_200
            },
            scrollbar: ThemeScrollbar {
                color: background_200,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            tooltip: ThemeTooltip {
                padding: 8.0,
                background_color: background_300,
            },
            loading_bar: ThemeLoadingBar {
                loading_bar_color: text_200,
                background_color: background_200,
            },
            text_accessory: ThemePaddingTextColorSpacing {
                padding: padding(4.0, 4.0, 4.0, 16.0),
                text_color: text_200,
                spacing: 8.0,
            },
            icon_accessory: ThemeIconAccessory {
                padding: padding(4.0, 4.0, 4.0, 16.0),
                icon_color: text_200,
            },
            hud: ThemeRoot {
                background_color: ThemeColor::new(0x1E1E1E, 0.7),
                border_radius: 30.0,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            hud_content: ThemePaddingOnly {
                padding: padding_axis(8.0, 16.0),
            },
        }
    }
}

fn init_theme(theme: GauntletComplexTheme) {
    THEME.set(theme).expect("already set");
}

fn get_theme() -> &'static GauntletComplexTheme {
    &THEME.get().expect("theme global var was not set")
}

static THEME: once_cell::sync::OnceCell<GauntletComplexTheme> = once_cell::sync::OnceCell::new();

const NOT_INTENDED_TO_BE_USED: ThemeColor = ThemeColor::new(0xAF5BFF, 1.0);

// keep colors more or less in sync with settings ui
const TRANSPARENT: ThemeColor = ThemeColor::new(0x000000, 0.0);

const fn padding(top: f32, right: f32, bottom: f32, left: f32) -> ThemePadding {
    ThemePadding::Each {
        top,
        right,
        bottom,
        left,
    }
}

const fn padding_all(value: f32) -> ThemePadding {
    ThemePadding::All {
        all: value
    }
}

const fn padding_axis(vertical: f32, horizontal: f32) -> ThemePadding {
    ThemePadding::Axis {
        vertical,
        horizontal,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeButton {
    padding: ThemePadding,
    background_color: ThemeColor,
    background_color_focused: ThemeColor,
    background_color_hovered: ThemeColor,
    text_color: ThemeColor,
    text_color_hovered: ThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCheckbox {
    background_color_checked: ThemeColor,
    background_color_unchecked: ThemeColor,

    background_color_checked_hovered: ThemeColor,
    background_color_unchecked_hovered: ThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,

    icon_color: ThemeColor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSelect {
    background_color: ThemeColor,
    background_color_hovered: ThemeColor,

    text_color: ThemeColor,
    text_color_hovered: ThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSelectMenu {
    background_color: ThemeColor,
    background_color_selected: ThemeColor,

    text_color: ThemeColor,
    text_color_selected: ThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTextField {
    background_color: ThemeColor,
    background_color_hovered: ThemeColor,

    text_color: ThemeColor,
    text_color_placeholder: ThemeColor,

    selection_color: ThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
    border_color_hovered: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSeparator {
    color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeScrollbar {
    color: ThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeRoot {
    background_color: ThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeActionShortcutModifier {
    padding: ThemePadding,
    spacing: f32,
    background_color: ThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeLoadingBar {
    loading_bar_color: ThemeColor,
    background_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeLink {
    text_color: ThemeColor,
    text_color_hovered: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCode {
    padding: ThemePadding,
    background_color: ThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInline {
    padding: ThemePadding,
    background_color: ThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDatePicker {
    background_color: ThemeColor,

    text_color: ThemeColor,
    text_color_selected: ThemeColor,
    text_color_hovered: ThemeColor,

    text_attenuated_color: ThemeColor,

    day_background_color: ThemeColor,
    day_background_color_selected: ThemeColor,
    day_background_color_hovered: ThemeColor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingTextColor {
    padding: ThemePadding,
    text_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingTextColorSize {
    padding: ThemePadding,
    text_color: ThemeColor,
    text_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingBackgroundColor {
    padding: ThemePadding,
    background_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBottomPanel {
    padding: ThemePadding,
    background_color: ThemeColor,
    spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTooltip {
    padding: f32, // TODO for some reason padding on tooltip is a single number in iced-rs
    background_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingOnly {
    padding: ThemePadding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeImage {
    padding: ThemePadding,

    border_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTextColor {
    text_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingSize {
    padding: ThemePadding,
    size: ExternalThemeSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalThemeGrid {
    padding: ThemePadding,
    spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeIconAccessory {
    padding: ThemePadding,
    icon_color: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingTextColorSpacing {
    padding: ThemePadding,
    text_color: ThemeColor,
    spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePaddingSpacing {
    padding: ThemePadding,
    spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalThemeSize {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThemePadding {
    Each {
        top: f32,
        right: f32,
        bottom: f32,
        left: f32,
    },
    Axis {
        vertical: f32,
        horizontal: f32,
    },
    All {
        all: f32,
    },
}

impl ThemePadding {
    fn to_iced(&self) -> Padding {
        match self {
            ThemePadding::Each { top, right, bottom, left } => {
                Padding {
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                    left: *left,
                }
            }
            ThemePadding::Axis { vertical, horizontal } => {
                Padding {
                    top: *vertical,
                    right: *horizontal,
                    bottom: *vertical,
                    left: *horizontal,
                }
            }
            ThemePadding::All { all } => {
                Padding {
                    top: *all,
                    right: *all,
                    bottom: *all,
                    left: *all,
                }
            }
        }
    }
}


#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct ThemeColor {
    r: u8,
    g: u8,
    b: u8,
    a: f32,
}

impl ThemeColor {
    #[allow(unused_parens)]
    const fn new(hex: u32, a: f32) -> Self {
        let r = ((hex & 0xff0000) >> 16) as u8;
        let g = ((hex & 0xff00) >> 8) as u8;
        let b = (hex & 0xff) as u8;

        Self { r, g, b, a }
    }

    fn from_rgba8(r: u8, g: u8, b: u8, a: f32) -> Self {
        let color = Color::from_rgba8(r, g, b, a);
        let [r, g, b, _] = color.into_rgba8();

        Self { r, g, b, a }
    }

    pub fn to_iced(&self) -> Color {
        Color::from_rgba8(self.r, self.g, self.b, self.a)
    }
}

pub trait ThemableWidget<'a, Message> {
    type Kind;

    fn themed(self, name: Self::Kind) -> Element<'a, Message>;
}

impl DefaultStyle for GauntletComplexTheme {
    fn default_style(&self) -> application::Appearance {
        let theme = get_theme();

        application::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.text.to_iced(),
        }
    }
}

#[cfg(target_os = "linux")]
impl iced_layershell::DefaultStyle for GauntletComplexTheme {
    fn default_style(&self) -> iced_layershell::Appearance {
        let theme = get_theme();

        iced_layershell::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.text.to_iced(),
        }
    }
}



