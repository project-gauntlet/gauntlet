use std::collections::HashMap;
use std::fmt::Display;

use gauntlet_common::model::CheckboxWidget;
use gauntlet_common::model::DatePickerWidget;
use gauntlet_common::model::FormWidget;
use gauntlet_common::model::FormWidgetOrderedMembers;
use gauntlet_common::model::PasswordFieldWidget;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::SelectWidget;
use gauntlet_common::model::SelectWidgetOrderedMembers;
use gauntlet_common::model::SeparatorWidget;
use gauntlet_common::model::TextFieldWidget;
use iced::Alignment;
use iced::Length;
use iced::advanced::text::Shaping;
use iced::alignment::Horizontal;
use iced::widget::Space;
use iced::widget::button;
use iced::widget::checkbox;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::text_input;
use iced_aw::date_picker;

use crate::ui::state::PluginViewState;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::date_picker::DatePickerStyle;
use crate::ui::theme::pick_list::PickListStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::text_input::TextInputStyle;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::state::CheckboxState;
use crate::ui::widget::state::DatePickerState;
use crate::ui::widget::state::RootState;
use crate::ui::widget::state::SelectState;
use crate::ui::widget::state::TextFieldState;

impl<'b> ComponentWidgets<'b> {
    fn render_text_field_widget<'a>(&self, widget: &TextFieldWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value, .. } = self.text_field_state(widget.__id__);

        text_input("", state_value)
            .on_input(move |value| ComponentWidgetEvent::OnChangeTextField { widget_id, value })
            .themed(TextInputStyle::FormInput)
    }

    fn render_password_field_widget<'a>(&self, widget: &PasswordFieldWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value, .. } = self.text_field_state(widget_id);

        text_input("", state_value)
            .secure(true)
            .on_input(move |value| ComponentWidgetEvent::OnChangePasswordField { widget_id, value })
            .themed(TextInputStyle::FormInput)
    }

    fn render_checkbox_widget<'a>(&self, widget: &CheckboxWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let CheckboxState { state_value } = self.checkbox_state(widget_id);

        checkbox(widget.title.as_deref().unwrap_or_default(), state_value.to_owned())
            .on_toggle(move |value| ComponentWidgetEvent::ToggleCheckbox { widget_id, value })
            .into()
    }

    fn render_date_picker_widget<'a>(&self, widget: &DatePickerWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let DatePickerState {
            state_value,
            show_picker,
        } = self.date_picker_state(widget.__id__);

        let button_text = text(state_value.to_string()).shaping(Shaping::Advanced);

        let button = button(button_text).on_press(ComponentWidgetEvent::ToggleDatePicker {
            widget_id: widget.__id__,
        });

        // TODO unable to customize buttons here, split to separate button styles
        //     DatePickerUnderlay,
        //     DatePickerOverlay,

        date_picker(
            show_picker.to_owned(),
            state_value.to_owned(),
            button,
            ComponentWidgetEvent::CancelDatePicker { widget_id },
            move |date| {
                ComponentWidgetEvent::SubmitDatePicker {
                    widget_id,
                    value: date.to_string(),
                }
            },
        )
        .themed(DatePickerStyle::Default)
    }

    fn render_select_widget<'a>(&self, widget: &SelectWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let SelectState { state_value } = self.select_state(widget_id);

        let items: Vec<_> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                match members {
                    SelectWidgetOrderedMembers::SelectItem(widget) => {
                        SelectItem {
                            value: widget.value.to_owned(),
                            label: widget.content.text.join(""),
                        }
                    }
                }
            })
            .collect();

        let state_value = state_value
            .clone()
            .map(|value| items.iter().find(|item| item.value == value))
            .flatten()
            .map(|value| value.clone());

        pick_list(items, state_value, move |item| {
            ComponentWidgetEvent::SelectPickList {
                widget_id,
                value: item.value,
            }
        })
        .themed(PickListStyle::Default)
    }

    fn render_separator_widget<'a>(&self, _widget: &SeparatorWidget) -> Element<'a, ComponentWidgetEvent> {
        horizontal_rule(1).into()
    }

    pub fn render_form_widget<'a>(
        &self,
        widget: &FormWidget,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let RootState { show_action_panel, .. } = self.root_state(widget_id);

        let items: Vec<Element<_>> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                fn render_field<'c, 'd>(
                    field: Element<'c, ComponentWidgetEvent>,
                    label: &'d Option<String>,
                ) -> Element<'c, ComponentWidgetEvent> {
                    let before_or_label: Element<_> = match label {
                        None => Space::with_width(Length::FillPortion(2)).into(),
                        Some(label) => {
                            let label: Element<_> = text(label.to_string())
                                .shaping(Shaping::Advanced)
                                .align_x(Horizontal::Right)
                                .width(Length::Fill)
                                .into();

                            container(label)
                                .width(Length::FillPortion(2))
                                .themed(ContainerStyle::FormInputLabel)
                        }
                    };

                    let form_input = container(field).width(Length::FillPortion(3)).into();

                    let after = Space::with_width(Length::FillPortion(2)).into();

                    let content = vec![before_or_label, form_input, after];

                    let row: Element<_> = row(content).align_y(Alignment::Center).themed(RowStyle::FormInput);

                    row
                }

                match members {
                    FormWidgetOrderedMembers::Separator(widget) => self.render_separator_widget(widget),
                    FormWidgetOrderedMembers::TextField(widget) => {
                        render_field(self.render_text_field_widget(widget), &widget.label)
                    }
                    FormWidgetOrderedMembers::PasswordField(widget) => {
                        render_field(self.render_password_field_widget(widget), &widget.label)
                    }
                    FormWidgetOrderedMembers::Checkbox(widget) => {
                        render_field(self.render_checkbox_widget(widget), &widget.label)
                    }
                    FormWidgetOrderedMembers::DatePicker(widget) => {
                        render_field(self.render_date_picker_widget(widget), &widget.label)
                    }
                    FormWidgetOrderedMembers::Select(widget) => {
                        render_field(self.render_select_widget(widget), &widget.label)
                    }
                }
            })
            .collect();

        let content: Element<_> = column(items).into();

        let content: Element<_> = container(content).width(Length::Fill).themed(ContainerStyle::FormInner);

        let content: Element<_> = scrollable(content).width(Length::Fill).into();

        let content: Element<_> = container(content).width(Length::Fill).themed(ContainerStyle::Form);

        self.render_plugin_root(
            *show_action_panel,
            widget_id,
            None,
            &None,
            &widget.content.actions,
            content,
            widget.is_loading.unwrap_or(false),
            plugin_view_state,
            entrypoint_name,
            action_shortcuts,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectItem {
    value: String,
    label: String,
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}
