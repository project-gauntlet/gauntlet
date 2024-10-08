use iced::{Padding, Renderer};
use iced::widget::Row;
use crate::ui::theme::{Element, GauntletTheme, get_theme, ThemableWidget};

pub enum RowStyle {
    ActionShortcut,
    FormInput,
    ListSectionTitle,
    GridSectionTitle,
    GridItemTitle,
    RootBottomPanel,
    RootBottomPanelPrimaryAction,
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
                    .spacing(theme.list_section_title.spacing)
            }
            RowStyle::GridSectionTitle => {
                self.padding(theme.grid_section_title.padding.to_iced())
                    .spacing(theme.grid_section_title.spacing)
            }
            RowStyle::GridItemTitle => {
                self.padding(theme.grid_item_title.padding.to_iced())
            }
            RowStyle::RootBottomPanel => {
                self.spacing(theme.root_bottom_panel.spacing)
            }
            RowStyle::RootBottomPanelPrimaryAction => {
                self.padding(Padding::from([0.0, theme.root_bottom_panel.spacing, 0.0, 0.0]))
            }
        }.into()
    }
}