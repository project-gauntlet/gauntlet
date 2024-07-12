use iced::{application, Color, Padding};

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

pub type Element<'a, Message> = iced::Element<'a, Message, GauntletTheme>;

#[derive(Debug, Clone)]
pub struct GauntletTheme {
    text: ExternalThemeColor,
    root: ExternalThemeRoot,
    action: ExternalThemeButton,
    action_panel: ExternalThemePaddingBackgroundColor,
    action_panel_title: ExternalThemePaddingOnly,
    action_shortcut: ExternalThemePaddingOnly,
    action_shortcut_modifier: ExternalThemeActionShortcutModifier,
    content_code_block: ExternalThemePaddingOnly,
    content_code_block_text: ExternalThemeCode,
    content_horizontal_break: ExternalThemePaddingOnly,
    content_image: ExternalThemeImage,
    content_paragraph: ExternalThemePaddingOnly,
    detail_content: ExternalThemePaddingOnly,
    detail_metadata: ExternalThemePaddingOnly,
    empty_view_image: ExternalThemePaddingSize,
    empty_view_subtitle: ExternalThemeTextColor,
    form: ExternalThemePaddingOnly,
    form_inner: ExternalThemePaddingOnly,
    form_input: ExternalThemePaddingOnly,
    form_input_label: ExternalThemePaddingOnly,
    form_input_date_picker: ExternalThemeDatePicker,
    form_input_date_picker_buttons: ExternalThemeButton,
    form_input_checkbox: ExternalThemeCheckbox,
    form_input_select: ExternalThemeSelect,
    form_input_select_menu: ExternalThemeSelectMenu,
    form_input_text_field: ExternalThemeTextField,
    grid: ExternalThemeGrid,
    grid_inner: ExternalThemePaddingOnly,
    list: ExternalThemePaddingOnly,
    list_inner: ExternalThemePaddingOnly,
    grid_item: ExternalThemeButton,
    grid_section_title: ExternalThemePaddingTextColor,
    inline: ExternalThemePaddingOnly,
    list_item: ExternalThemeButton,
    list_item_subtitle: ExternalThemePaddingTextColor,
    list_item_title: ExternalThemePaddingOnly,
    list_item_icon: ExternalThemePaddingOnly,
    list_section_title: ExternalThemePaddingTextColor,
    main_list: ExternalThemePaddingOnly,
    main_list_inner: ExternalThemePaddingOnly,
    main_list_item: ExternalThemeButton,
    main_list_item_icon: ExternalThemePaddingOnly,
    main_list_item_sub_text: ExternalThemePaddingTextColor,
    main_list_item_text: ExternalThemePaddingOnly,
    main_search_bar: ExternalThemePaddingOnly,
    metadata_item_value: ExternalThemePaddingOnly,
    metadata_content_inner: ExternalThemePaddingOnly,
    metadata_inner: ExternalThemePaddingOnly,
    metadata_separator: ExternalThemePaddingOnly,
    metadata_tag_item: ExternalThemePaddingOnly,
    metadata_item_label: ExternalThemePaddingTextColorSize,
    metadata_link_icon: ExternalThemePaddingOnly,
    metadata_tag_item_button: ExternalThemeButton,
    plugin_error_view_description: ExternalThemePaddingOnly,
    plugin_error_view_title: ExternalThemePaddingOnly,
    preference_required_view_description: ExternalThemePaddingOnly,
    root_bottom_panel: ExternalThemePaddingBackgroundColor,
    root_bottom_panel_action_button: ExternalThemeButton,
    root_content: ExternalThemePaddingOnly,
    root_top_panel: ExternalThemePaddingOnly,
    root_top_panel_button: ExternalThemeButton,
    metadata_link: ExternalThemeLink,
    separator: ExternalThemeSeparator,
    scrollbar: ExternalThemeScrollbar,
    tooltip: ExternalThemeTooltip,
}

impl Default for GauntletTheme {
    fn default() -> Self {
        unreachable!()
    }
}

// TODO add border on focus, lighter background on hover
// TODO padding on button is padding, not margin, a lot of margins missing?

impl GauntletTheme {
    pub fn new() -> Self {
        let theme = Self {
            text: TEXT,
            root: ExternalThemeRoot {
                background_color: BACKGROUND,
                border_radius: 10.0,
                border_width: 1.0,
                border_color: BACKGROUND_LIGHT,
            },
            action_panel: ExternalThemePaddingBackgroundColor {
                padding: padding_all(8.0),
                background_color: BACKGROUND_OVERLAY,
            },
            action_panel_title: ExternalThemePaddingOnly {
                padding: padding(2.0, 8.0, 4.0, 8.0),
            },
            action: ExternalThemeButton {
                padding: padding_all(8.0),
                background_color: TRANSPARENT,
                background_color_hovered: BACKGROUND_LIGHT,
                text_color: TEXT,
                text_color_hovered: TEXT,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            action_shortcut: ExternalThemePaddingOnly {
                padding: padding_all(0.0)
            },
            action_shortcut_modifier: ExternalThemeActionShortcutModifier {
                padding: padding_axis(0.0, 4.0),
                spacing: 4.0,
                background_color: BACKGROUND_LIGHTER,
                border_radius: 4.0,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            form_input: ExternalThemePaddingOnly {
                padding: padding_all(8.0)
            },
            metadata_tag_item: ExternalThemePaddingOnly {
                padding: padding(0.0, 8.0, 4.0, 0.0),
            },
            metadata_tag_item_button: ExternalThemeButton {
                padding: padding_axis(2.0, 8.0),
                background_color: PRIMARY,
                background_color_hovered: PRIMARY_HOVERED,
                text_color: TEXT_DARK,
                text_color_hovered: TEXT_DARK,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            metadata_item_label: ExternalThemePaddingTextColorSize {
                padding: padding_all(0.0),
                text_color: TEXT_DARKER,
                text_size: 14.0,
            },
            metadata_item_value: ExternalThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            metadata_link_icon: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 4.0),
            },
            root_bottom_panel: ExternalThemePaddingBackgroundColor {
                padding: padding_axis(4.0, 8.0),
                background_color: BACKGROUND_OVERLAY,
            },
            root_top_panel: ExternalThemePaddingOnly {
                padding: padding_all(12.0),
            },
            list_item_subtitle: ExternalThemePaddingTextColor {
                padding: padding_all(4.0),
                text_color: TEXT_DARKER,
            },
            list_item_title: ExternalThemePaddingOnly {
                padding: padding_all(4.0),
            },
            content_paragraph: ExternalThemePaddingOnly {
                padding: padding_all(8.0)
            },
            content_code_block: ExternalThemePaddingOnly {
                padding: padding_all(0.0),
            },
            content_image: ExternalThemeImage {
                padding: padding_all(0.0),
                border_radius: 6.0,
            },
            inline: ExternalThemePaddingOnly {
                padding: padding_all(8.0)
            },
            empty_view_image: ExternalThemePaddingSize {
                padding: padding_all(8.0),
                size: ExternalThemeSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            grid_item: ExternalThemeButton {
                padding: padding_all(8.0),
                background_color: BACKGROUND_LIGHT,
                background_color_hovered: BACKGROUND_LIGHTER,
                text_color: TEXT,
                text_color_hovered: TEXT,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            content_horizontal_break: ExternalThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            content_code_block_text: ExternalThemeCode {
                padding: padding_axis(4.0, 8.0),
                background_color: BACKGROUND_LIGHT,
                border_radius: 4.0,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            metadata_separator: ExternalThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            root_top_panel_button: ExternalThemeButton {
                padding: padding_axis(3.0, 5.0),
                background_color: BACKGROUND_LIGHT,
                background_color_hovered: BACKGROUND_LIGHTER,
                text_color: TEXT,
                text_color_hovered: TEXT,
                border_radius: 6.0,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            root_bottom_panel_action_button: ExternalThemeButton {
                padding: padding_axis(3.0, 5.0),
                background_color: BACKGROUND_LIGHT,
                background_color_hovered: BACKGROUND_LIGHTER,
                text_color: TEXT,
                text_color_hovered: TEXT,
                border_radius: 6.0,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            list_item: ExternalThemeButton {
                padding: padding_all(5.0),
                background_color: TRANSPARENT,
                background_color_hovered: BACKGROUND_LIGHT,
                text_color: TEXT,
                text_color_hovered: TEXT,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            list_item_icon: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 4.0)
            },
            root_content: ExternalThemePaddingOnly {
                padding: padding_all(0.0), // TODO hardcode this?
            },
            detail_metadata: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 12.0),
            },
            metadata_inner: ExternalThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            detail_content: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 12.0),
            },
            metadata_content_inner: ExternalThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            form: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 12.0),
            },
            form_inner: ExternalThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            grid: ExternalThemeGrid {
                spacing: 8.0,
                padding: padding_axis(0.0, 12.0),
            },
            grid_inner: ExternalThemePaddingOnly {
                padding: padding_axis(12.0, 0.0),
            },
            list: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 8.0),
            },
            list_inner: ExternalThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            form_input_label: ExternalThemePaddingOnly {
                padding: padding_axis(4.0, 12.0),
            },
            list_section_title: ExternalThemePaddingTextColor {
                padding: padding_axis(4.0, 8.0),
                text_color: TEXT_DARKER,
            },
            grid_section_title: ExternalThemePaddingTextColor {
                padding: padding_axis(4.0, 0.0),
                text_color: TEXT_DARKER,
            },
            main_list_item: ExternalThemeButton {
                padding: padding_all(5.0),
                background_color: TRANSPARENT,
                background_color_hovered: BACKGROUND_LIGHT,
                text_color: TEXT,
                text_color_hovered: TEXT,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            main_list_item_text: ExternalThemePaddingOnly {
                padding: padding_all(4.0),
            },
            main_list_item_sub_text: ExternalThemePaddingTextColor {
                padding: padding_axis(4.0, 12.0),
                text_color: TEXT_DARKER,
            },
            main_list_item_icon: ExternalThemePaddingOnly {
                padding: padding(0.0, 7.0, 0.0, 5.0),
            },
            main_list: ExternalThemePaddingOnly {
                padding: padding_axis(0.0, 8.0),
            },
            main_list_inner: ExternalThemePaddingOnly {
                padding: padding_axis(8.0, 0.0),
            },
            main_search_bar: ExternalThemePaddingOnly {
                padding: padding_all(12.0),
            },
            plugin_error_view_title: ExternalThemePaddingOnly {
                padding: padding_all(12.0),
            },
            plugin_error_view_description: ExternalThemePaddingOnly {
                padding: padding_all(12.0),
            },
            preference_required_view_description: ExternalThemePaddingOnly {
                padding: padding_all(12.0),
            },
            metadata_link: ExternalThemeLink {
                text_color: TEXT,
                text_color_hovered: TEXT_HOVERED,
            },
            empty_view_subtitle: ExternalThemeTextColor {
                text_color: TEXT_DARKER,
            },
            form_input_date_picker: ExternalThemeDatePicker {
                background_color: BACKGROUND,
                border_radius: 10.0,
                border_width: 1.0,
                border_color: BACKGROUND_LIGHT,
                text_color: TEXT,
                text_color_selected: TEXT_DARKER,
                text_color_hovered: TEXT_DARKER,
                text_attenuated_color: ExternalThemeColor::new(0xCAC2B6, 0.3),
                day_background_color: BACKGROUND_LIGHT,
                day_background_color_selected: BACKGROUND_LIGHT,
                day_background_color_hovered: BACKGROUND_LIGHT,
            },
            form_input_date_picker_buttons: ExternalThemeButton {
                padding: padding_all(8.0),
                background_color: PRIMARY,
                background_color_hovered: PRIMARY_HOVERED,
                text_color: TEXT_DARK,
                text_color_hovered: TEXT_DARK,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            form_input_checkbox: ExternalThemeCheckbox {
                background_color_checked: PRIMARY,
                background_color_unchecked: BACKGROUND,
                background_color_checked_hovered: PRIMARY_HOVERED,
                background_color_unchecked_hovered: BACKGROUND_LIGHT,
                border_radius: 4.0,
                border_width: 1.0,
                border_color: PRIMARY,
                icon_color: BACKGROUND,
            },
            form_input_select: ExternalThemeSelect {
                background_color: PRIMARY,
                background_color_hovered: PRIMARY_HOVERED,
                text_color: TEXT_DARK,
                text_color_hovered: TEXT_DARK,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 1.0,
                border_color: BACKGROUND_LIGHT,
            },
            form_input_select_menu: ExternalThemeSelectMenu {
                background_color: BACKGROUND,
                background_color_selected: BACKGROUND_LIGHT,
                text_color: TEXT,
                text_color_selected: TEXT,
                border_radius: BUTTON_BORDER_RADIUS,
                border_width: 1.0,
                border_color: BACKGROUND_LIGHT,
            },
            form_input_text_field: ExternalThemeTextField {
                background_color: TRANSPARENT,
                background_color_hovered: BACKGROUND_LIGHT,
                text_color: TEXT,
                text_color_placeholder: TEXT_DARKER,
                selection_color: BACKGROUND_LIGHT,
                border_radius: 4.0,
                border_width: 1.0,
                border_color: BACKGROUND_LIGHT,
                border_color_hovered: BACKGROUND_LIGHT,
            },
            separator: ExternalThemeSeparator {
                color: BACKGROUND_LIGHT
            },
            scrollbar: ExternalThemeScrollbar {
                color: PRIMARY,
                border_radius: 4.0,
                border_width: 0.0,
                border_color: TRANSPARENT,
            },
            tooltip: ExternalThemeTooltip {
                padding: 8.0,
                background_color: BACKGROUND_OVERLAY,
            },
        };

        init_theme(theme.clone());

        theme
    }
}

fn init_theme(theme: GauntletTheme) {
    THEME.set(theme).expect("already set");
}

fn get_theme() -> &'static GauntletTheme {
    &THEME.get().expect("theme global var was not set")
}

static THEME: once_cell::sync::OnceCell<GauntletTheme> = once_cell::sync::OnceCell::new();

const NOT_INTENDED_TO_BE_USED: ExternalThemeColor = ExternalThemeColor::new(0xAF5BFF, 1.0);

// keep colors more or less in sync with settings ui
const TRANSPARENT: ExternalThemeColor = ExternalThemeColor::new(0x000000, 0.0);
const BACKGROUND: ExternalThemeColor = ExternalThemeColor::new(0x2C323A, 1.0);
const BACKGROUND_OVERLAY: ExternalThemeColor = ExternalThemeColor::new(0x333a42, 1.0);
const BACKGROUND_LIGHT: ExternalThemeColor = ExternalThemeColor::new(0x48505B, 0.5);
const BACKGROUND_LIGHTER: ExternalThemeColor = ExternalThemeColor::new(0x626974, 0.3);
const TEXT: ExternalThemeColor = ExternalThemeColor::new(0xCAC2B6, 1.0);
const TEXT_HOVERED: ExternalThemeColor = ExternalThemeColor::new(0xE1E0DD, 1.0);
const TEXT_DARKER: ExternalThemeColor = ExternalThemeColor::new(0x848484, 1.0);
const TEXT_DARK: ExternalThemeColor = ExternalThemeColor::new(0x1D242C, 1.0);
const PRIMARY: ExternalThemeColor = ExternalThemeColor::new(0xC79F60, 1.0);
const PRIMARY_HOVERED: ExternalThemeColor = ExternalThemeColor::new(0xD7B37A, 1.0);

const BUTTON_BORDER_RADIUS: f32 = 6.0;

const fn padding(top: f32, right: f32, bottom: f32, left: f32) -> ExternalThemePadding {
    ExternalThemePadding {
        top,
        right,
        bottom,
        left,
    }
}

const fn padding_all(value: f32) -> ExternalThemePadding {
    ExternalThemePadding {
        top: value,
        right: value,
        bottom: value,
        left: value,
    }
}

const fn padding_axis(vertical: f32, horizontal: f32) -> ExternalThemePadding {
    ExternalThemePadding {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

#[derive(Debug, Clone)]
pub struct ExternalThemeButton {
    padding: ExternalThemePadding,
    background_color: ExternalThemeColor,
    background_color_hovered: ExternalThemeColor,
    text_color: ExternalThemeColor,
    text_color_hovered: ExternalThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeCheckbox {
    background_color_checked: ExternalThemeColor,
    background_color_unchecked: ExternalThemeColor,

    background_color_checked_hovered: ExternalThemeColor,
    background_color_unchecked_hovered: ExternalThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,

    icon_color: ExternalThemeColor
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSelect {
    background_color: ExternalThemeColor,
    background_color_hovered: ExternalThemeColor,

    text_color: ExternalThemeColor,
    text_color_hovered: ExternalThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSelectMenu {
    background_color: ExternalThemeColor,
    background_color_selected: ExternalThemeColor,

    text_color: ExternalThemeColor,
    text_color_selected: ExternalThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeTextField {
    background_color: ExternalThemeColor,
    background_color_hovered: ExternalThemeColor,

    text_color: ExternalThemeColor,
    text_color_placeholder: ExternalThemeColor,

    selection_color: ExternalThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
    border_color_hovered: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSeparator {
    color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeScrollbar {
    color: ExternalThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeRoot {
    background_color: ExternalThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeActionShortcutModifier {
    padding: ExternalThemePadding,
    spacing: f32,
    background_color: ExternalThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeLink {
    text_color: ExternalThemeColor,
    text_color_hovered: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeCode {
    padding: ExternalThemePadding,
    background_color: ExternalThemeColor,
    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeDatePicker {
    background_color: ExternalThemeColor,

    border_radius: f32,
    border_width: f32,
    border_color: ExternalThemeColor,

    text_color: ExternalThemeColor,
    text_color_selected: ExternalThemeColor,
    text_color_hovered: ExternalThemeColor,

    text_attenuated_color: ExternalThemeColor,

    day_background_color: ExternalThemeColor,
    day_background_color_selected: ExternalThemeColor,
    day_background_color_hovered: ExternalThemeColor
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingTextColor {
    padding: ExternalThemePadding,
    text_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingTextColorSize {
    padding: ExternalThemePadding,
    text_color: ExternalThemeColor,
    text_size: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingBackgroundColor {
    padding: ExternalThemePadding,
    background_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeTooltip {
    padding: f32, // TODO for some reason padding on tooltip is a single number in iced-rs
    background_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingOnly {
    padding: ExternalThemePadding,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeImage {
    padding: ExternalThemePadding,

    border_radius: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeTextColor {
    text_color: ExternalThemeColor,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingSize {
    padding: ExternalThemePadding,
    size: ExternalThemeSize,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeGrid {
    padding: ExternalThemePadding,
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSize {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePadding {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl ExternalThemePadding {
    fn to_iced(&self) -> Padding {
        Padding {
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            left: self.left,
        }
    }
}


#[derive(Clone, Debug)]
pub struct ExternalThemeColor {
    hex: u32,
    a: f32,
}

impl ExternalThemeColor {
    const fn new(hex: u32, a: f32) -> Self {
        Self { hex, a }
    }

    #[allow(unused_parens)]
    pub fn to_iced(&self) -> Color {
        let hex = self.hex;
        let r = (hex & 0xff0000) >> 16;
        let g = (hex & 0xff00) >> 8;
        let b = (hex & 0xff);

        Color::from_rgba8(r as u8, g as u8, b as u8, self.a)
    }
}

pub trait ThemableWidget<'a, Message> {
    type Kind;

    fn themed(self, name: Self::Kind) -> Element<'a, Message>;
}

impl application::StyleSheet for GauntletTheme {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> application::Appearance {
        let theme = get_theme();

        application::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.text.to_iced(),
        }
    }
}



