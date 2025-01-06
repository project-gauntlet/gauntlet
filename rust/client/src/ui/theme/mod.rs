use arc_swap::{ArcSwap, Guard};
use gauntlet_common::model::{UiTheme, UiThemeColor, UiThemeMode};
use iced::application::DefaultStyle;
use iced::{application, Color, Padding};
use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub struct GauntletComplexTheme {
    text: Color,
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

// TODO padding on button is padding, not margin, a lot of margins missing?

impl GauntletComplexTheme {
    pub fn set_global(theme: GauntletComplexTheme) {
        init_theme(theme);
    }

    pub fn update_global(theme: GauntletComplexTheme) {
        set_theme(theme);
    }

    pub fn new(simple_theme: UiTheme) -> GauntletComplexTheme {
        let UiTheme {
            mode,
            background,
            text,
            window,
            content
        } = simple_theme;

        let [background_100, background_200, background_300, background_400] = background;
        let [text_100, text_200, text_300, text_400] = text;

        fn to_iced(ui_color: &UiThemeColor) -> Color {
            Color::from_rgba(ui_color.r, ui_color.g, ui_color.b, ui_color.a)
        }

        let [background_100, background_200, background_300, background_400] = [
            to_iced(&background_100),
            to_iced(&background_200),
            to_iced(&background_300),
            to_iced(&background_400)
        ];
        let [text_100, text_200, text_300, _text_400] = [
            to_iced(&text_100),
            to_iced(&text_200),
            to_iced(&text_300),
            to_iced(&text_400)
        ];

        GauntletComplexTheme {
            text: text_100,
            root: ThemeRoot {
                background_color: background_400,
                #[cfg(not(target_os = "macos"))]
                border_radius: window.border.radius,
                #[cfg(not(target_os = "macos"))]
                border_width: window.border.width,
                #[cfg(not(target_os = "macos"))]
                border_color: to_iced(&window.border.color),
                #[cfg(target_os = "macos")]
                border_radius: 0.0,
                #[cfg(target_os = "macos")]
                border_width: 0.0,
                #[cfg(target_os = "macos")]
                border_color: Color::TRANSPARENT,
            },
            popup: ThemeRoot {
                background_color: background_400,
                border_radius: window.border.radius,
                border_width: window.border.width,
                border_color: to_iced(&window.border.color),
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
                background_color: Color::TRANSPARENT,
                background_color_focused: background_100,
                background_color_hovered: background_300,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                border_color: Color::TRANSPARENT,
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
                    UiThemeMode::Light => background_300,
                    UiThemeMode::Dark => background_200
                },
                background_color_focused: match mode {
                    UiThemeMode::Light => background_200,
                    UiThemeMode::Dark => background_100
                },
                background_color_hovered: match mode {
                    UiThemeMode::Light => background_200,
                    UiThemeMode::Dark => background_100
                },
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                border_color: Color::TRANSPARENT,
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
                border_color: Color::TRANSPARENT,
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
                border_color: Color::TRANSPARENT,
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
                    UiThemeMode::Light => background_300,
                    UiThemeMode::Dark => background_200
                },
                background_color_focused: match mode {
                    UiThemeMode::Light => background_200,
                    UiThemeMode::Dark => background_100
                },
                background_color_hovered: match mode {
                    UiThemeMode::Light => background_200,
                    UiThemeMode::Dark => background_100
                },
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            root_bottom_panel: ThemeBottomPanel {
                padding: padding_axis(6.0, 8.0),
                background_color: background_300,
                spacing: 8.0
            },
            root_bottom_panel_action_toggle_button: ThemeButton {
                padding: padding_axis(3.0, 5.0),
                background_color: Color::TRANSPARENT,
                background_color_focused: background_200,
                background_color_hovered: background_200,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                background_color: Color::TRANSPARENT,
                background_color_focused: background_200,
                background_color_hovered: background_300,
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                background_color: Color::TRANSPARENT,
                background_color_focused: match mode {
                    UiThemeMode::Light => background_300,
                    UiThemeMode::Dark => background_200
                },
                background_color_hovered: match mode {
                    UiThemeMode::Light => background_200,
                    UiThemeMode::Dark => background_300
                },
                text_color: text_100,
                text_color_hovered: text_100,
                border_radius: content.border.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                border_color: Color::TRANSPARENT,
            },
            form_input_checkbox: ThemeCheckbox {
                background_color_checked: text_200,
                background_color_unchecked: Color::TRANSPARENT,
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
                background_color: Color::TRANSPARENT,
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
                border_color: Color::TRANSPARENT,
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
                background_color: Color::from_rgba8(30,30,30, 0.7),
                border_radius: 30.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            hud_content: ThemePaddingOnly {
                padding: padding_axis(8.0, 16.0),
            },
        }
    }
}

fn init_theme(theme: GauntletComplexTheme) {
    THEME.set(ArcSwap::new(Arc::new(theme))).expect("already set");
}

fn set_theme(theme: GauntletComplexTheme) {
    THEME.get().expect("theme global var was not set").store(Arc::new(theme))
}

fn get_theme() -> Guard<Arc<GauntletComplexTheme>> {
    THEME
        .get()
        .expect("theme global var was not set")
        .load()
}

static THEME: once_cell::sync::OnceCell<ArcSwap<GauntletComplexTheme>> = once_cell::sync::OnceCell::new();

const NOT_INTENDED_TO_BE_USED: Color = Color::from_rgba(175.0 / 255.0, 91.0 / 255.0, 255.0 / 255.0, 1.0);

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

#[derive(Debug, Clone)]
pub struct ThemeButton {
    padding: ThemePadding,
    background_color: Color,
    background_color_focused: Color,
    background_color_hovered: Color,
    text_color: Color,
    text_color_hovered: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeCheckbox {
    background_color_checked: Color,
    background_color_unchecked: Color,

    background_color_checked_hovered: Color,
    background_color_unchecked_hovered: Color,

    border_radius: f32,
    border_width: f32,
    border_color: Color,

    icon_color: Color
}

#[derive(Debug, Clone)]
pub struct ThemeSelect {
    background_color: Color,
    background_color_hovered: Color,

    text_color: Color,
    text_color_hovered: Color,

    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeSelectMenu {
    background_color: Color,
    background_color_selected: Color,

    text_color: Color,
    text_color_selected: Color,

    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeTextField {
    background_color: Color,
    background_color_hovered: Color,

    text_color: Color,
    text_color_placeholder: Color,

    selection_color: Color,

    border_radius: f32,
    border_width: f32,
    border_color: Color,
    border_color_hovered: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeSeparator {
    color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeScrollbar {
    color: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeRoot {
    background_color: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeActionShortcutModifier {
    padding: ThemePadding,
    spacing: f32,
    background_color: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeLoadingBar {
    loading_bar_color: Color,
    background_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeLink {
    text_color: Color,
    text_color_hovered: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeCode {
    padding: ThemePadding,
    background_color: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeInline {
    padding: ThemePadding,
    background_color: Color,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeDatePicker {
    background_color: Color,

    text_color: Color,
    text_color_selected: Color,
    text_color_hovered: Color,

    text_attenuated_color: Color,

    day_background_color: Color,
    day_background_color_selected: Color,
    day_background_color_hovered: Color
}

#[derive(Debug, Clone)]
pub struct ThemePaddingTextColor {
    padding: ThemePadding,
    text_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemePaddingTextColorSize {
    padding: ThemePadding,
    text_color: Color,
    text_size: f32,
}

#[derive(Debug, Clone)]
pub struct ThemePaddingBackgroundColor {
    padding: ThemePadding,
    background_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeBottomPanel {
    padding: ThemePadding,
    background_color: Color,
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeTooltip {
    padding: f32, // TODO for some reason padding on tooltip is a single number in iced-rs
    background_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemePaddingOnly {
    padding: ThemePadding,
}

#[derive(Debug, Clone)]
pub struct ThemeImage {
    padding: ThemePadding,

    border_radius: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeTextColor {
    text_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemePaddingSize {
    padding: ThemePadding,
    size: ExternalThemeSize,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeGrid {
    padding: ThemePadding,
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ThemeIconAccessory {
    padding: ThemePadding,
    icon_color: Color,
}

#[derive(Debug, Clone)]
pub struct ThemePaddingTextColorSpacing {
    padding: ThemePadding,
    text_color: Color,
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ThemePaddingSpacing {
    padding: ThemePadding,
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSize {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
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

pub trait ThemableWidget<'a, Message> {
    type Kind;

    fn themed(self, name: Self::Kind) -> Element<'a, Message>;
}

impl DefaultStyle for GauntletComplexTheme {
    fn default_style(&self) -> application::Appearance {
        let theme = get_theme();

        application::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.text,
        }
    }
}

#[cfg(target_os = "linux")]
impl iced_layershell::DefaultStyle for GauntletComplexTheme {
    fn default_style(&self) -> iced_layershell::Appearance {
        let theme = get_theme();

        iced_layershell::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.text,
        }
    }
}

