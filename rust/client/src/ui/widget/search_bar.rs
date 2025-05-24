use gauntlet_common::model::SearchBarWidget;
use iced::widget::text_input;

use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::text_input::TextInputStyle;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::state::TextFieldState;

impl<'b> ComponentWidgets<'b> {
    pub fn render_search_bar_widget<'a>(&self, widget: &SearchBarWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState {
            state_value,
            text_input_id,
        } = self.text_field_state(widget_id);

        text_input(widget.placeholder.as_deref().unwrap_or_default(), state_value)
            .id(text_input_id.clone())
            .ignore_with_modifiers(true)
            .on_input(move |value| ComponentWidgetEvent::OnChangeSearchBar { widget_id, value })
            .themed(TextInputStyle::PluginSearchBar)
    }
}
