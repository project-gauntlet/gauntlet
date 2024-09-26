use iced::application::{Appearance, StyleSheet};

pub mod container;
pub mod text;
pub mod table;
pub mod button;
pub mod text_input;
pub mod number_input;
pub mod rule;
pub mod checkbox;
pub mod pick_list;
pub mod scrollable;
pub mod shortcut_selector;
mod spinner;

pub type Element<'a, Message> = iced::Element<'a, Message, GauntletSettingsTheme>;

#[derive(Default)]
pub struct GauntletSettingsTheme;

impl StyleSheet for GauntletSettingsTheme {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background_color: BACKGROUND_DARKEST.to_iced(),
            text_color: TEXT_LIGHTEST.to_iced(),
        }
    }
}

// keep colors more or less in sync with main ui
pub const NOT_INTENDED_TO_BE_USED: ThemeColor = ThemeColor::new(0xAF5BFF, 1.0);

pub const TRANSPARENT: ThemeColor = ThemeColor::new(0x000000, 0.0);
pub const BACKGROUND_LIGHTEST: ThemeColor = ThemeColor::new(0x626974, 0.3);
pub const BACKGROUND_LIGHTER: ThemeColor = ThemeColor::new(0x48505B, 0.5);
pub const BACKGROUND_DARKER: ThemeColor = ThemeColor::new(0x333a42, 1.0);
pub const BACKGROUND_DARKEST: ThemeColor = ThemeColor::new(0x2C323A, 1.0);
pub const TEXT_LIGHTEST: ThemeColor = ThemeColor::new(0xDDDFE1, 1.0);
pub const TEXT_LIGHTER: ThemeColor = ThemeColor::new(0x9AA0A6, 1.0);
pub const TEXT_DARKER: ThemeColor = ThemeColor::new(0x6B7785, 1.0);
pub const TEXT_DARKEST: ThemeColor = ThemeColor::new(0x1D242C, 1.0);
pub const PRIMARY: ThemeColor = ThemeColor::new(0xC79F60, 1.0);
pub const PRIMARY_HOVERED: ThemeColor = ThemeColor::new(0xD7B37A, 1.0);

pub const BUTTON_BORDER_RADIUS: f32 = 6.0;

// settings specific colors
pub const SUCCESS: ThemeColor = ThemeColor::new(0x659B5E, 1.0);
pub const DANGER: ThemeColor = ThemeColor::new(0x6C1B1B, 1.0);
pub const DANGER_BRIGHT: ThemeColor = ThemeColor::new(0xC20000, 1.0);


#[derive(Clone, Debug)]
pub struct ThemeColor {
    hex: u32,
    a: f32,
}

impl ThemeColor {
    const fn new(hex: u32, a: f32) -> Self {
        Self { hex, a }
    }

    #[allow(unused_parens)]
    pub fn to_iced(&self) -> iced::Color {
        let hex = self.hex;
        let r = (hex & 0xff0000) >> 16;
        let g = (hex & 0xff00) >> 8;
        let b = (hex & 0xff);

        iced::Color::from_rgba8(r as u8, g as u8, b as u8, self.a)
    }
}
