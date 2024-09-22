use iced::Renderer;
use iced::widget::Row;
use crate::ui::theme::{Element, GauntletTheme, get_theme, ThemableWidget};

pub enum RowStyle {
    ActionShortcut,
    FormInput,
    ListSectionTitle,
    GridSectionTitle,
    GridItemTitle,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Row<'a, Message, GauntletTheme, Renderer> {
    type Kind = RowStyle;

    fn themed(self, name: RowStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match name {
            RowStyle::ActionShortcut => {
                self.padding(theme.action_shortcut.padding.to_iced())
            }
            RowStyle::FormInput => {
                self.padding(theme.form_input.padding.to_iced())
            }
            RowStyle::ListSectionTitle => {
                self.padding(theme.list_section_title.padding.to_iced())
            }
            RowStyle::GridSectionTitle => {
                self.padding(theme.grid_section_title.padding.to_iced())
            }
            RowStyle::GridItemTitle => {
                self.padding(theme.grid_item_title.padding.to_iced())
            }
        }.into()
    }
}