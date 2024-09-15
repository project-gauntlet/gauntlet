use std::collections::HashMap;
use std::fmt::Display;
use iced::{Length, Padding};
use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced_aw::core::icons;
use iced_aw::number_input;
use common::model::{EntrypointId, PluginId, PluginPreference};
use crate::theme::button::ButtonStyle;
use crate::theme::Element;
use crate::theme::text::TextStyle;
use crate::views::plugins::PluginPreferenceUserDataState;

#[derive(Debug, Clone)]
pub enum PluginPreferencesMsg {
    UpdatePreferenceValue {
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        id: String,
        user_data: PluginPreferenceUserDataState
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectItem { // TODO private
    value: String,
    label: String
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub fn preferences_ui<'a>(
    plugin_id: PluginId,
    entrypoint_id: Option<EntrypointId>,
    preferences: &HashMap<String, PluginPreference>,
    preference_user_data: &HashMap<(PluginId, Option<EntrypointId>, String), PluginPreferenceUserDataState>
) -> Element<'a, PluginPreferencesMsg> {
    let mut column_content = vec![];

    let mut preferences: Vec<_> = preferences.iter()
        .map(|entry| entry)
        .collect();

    preferences.sort_by_key(|(&ref key, _)| key);

    for (preference_id, preference) in preferences {
        let plugin_id = plugin_id.clone();
        let entrypoint_id = entrypoint_id.clone();

        let user_data = preference_user_data.get(&(plugin_id.clone(), entrypoint_id.clone(), preference_id.clone()));

        let (preference_name, description) = match preference {
            PluginPreference::Number { name, description, .. } => (name, description),
            PluginPreference::String { name, description, .. } => (name, description),
            PluginPreference::Enum { name, description, .. } => (name, description),
            PluginPreference::Bool { name, description, .. } => (name, description),
            PluginPreference::ListOfStrings { name, description, .. } => (name, description),
            PluginPreference::ListOfNumbers { name, description, .. } => (name, description),
            PluginPreference::ListOfEnums { name, description, .. } => (name, description),
        };

        let preference_id = preference_id.to_owned();
        let preference_name = preference_name.to_owned();
        let description = description.to_owned();

        let preference_label: Element<_> = text(&preference_name)
            .size(14)
            .style(TextStyle::Subtitle)
            .into();

        let preference_label = container(preference_label)
            .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
            .into();

        column_content.push(preference_label);

        if !description.trim().is_empty() {
            let description = container(text(description))
                .padding(Padding::from([4.0, 8.0]))
                .into();

            column_content.push(description);
        }

        let input_fields: Vec<Element<_>> = match preference {
            PluginPreference::Number { default, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserDataState::Number { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let value = value.or(default.to_owned()).unwrap_or_default();

                let input_field: Element<_> = number_input(value, f64::MAX, std::convert::identity)
                    .bounds((f64::MIN, f64::MAX))
                    .width(Length::Fill)
                    .into();

                let input_field = input_field.map(Box::new(move |value| {
                    PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::Number {
                            value: Some(value),
                        },
                    }
                }));

                let input_field = container(input_field)
                    .width(Length::Fill)
                    .padding(Padding::from([4.0, 8.0]))
                    .into();

                vec![input_field]
            }
            PluginPreference::String { default, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserDataState::String { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let default = default.to_owned().unwrap_or_default();

                let input_field: Element<_> = text_input(&default, &value.unwrap_or_default())
                    .on_input(Box::new(move |value| {
                        PluginPreferencesMsg::UpdatePreferenceValue {
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                            id: preference_id.to_owned(),
                            user_data: PluginPreferenceUserDataState::String {
                                value: Some(value),
                            },
                        }
                    }))
                    .into();

                let input_field = container(input_field)
                    .padding(Padding::new(8.0))
                    .into();

                vec![input_field]
            }
            PluginPreference::Enum { default, enum_values, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserDataState::Enum { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let enum_values: Vec<_> = enum_values.iter()
                    .map(|enum_item| SelectItem { label: enum_item.label.to_owned(), value: enum_item.value.to_owned() })
                    .collect();

                let value = value.or(default.to_owned())
                    .map(|value| enum_values.iter().find(|item| item.value == value))
                    .flatten()
                    .map(|value| value.clone());

                let input_field: Element<_> = pick_list(
                    enum_values,
                    value,
                    Box::new(move |item: SelectItem| PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::Enum {
                            value: Some(item.value),
                        },
                    })
                )
                    .width(Length::Fill)
                    .into();

                let input_field = container(input_field)
                    .padding(Padding::new(8.0))
                    .width(Length::Fill)
                    .into();

                vec![input_field]
            }
            PluginPreference::Bool { default, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserDataState::Bool { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let input_field: Element<_> = checkbox(preference_name.clone(), value.or(default.to_owned()).unwrap_or(false))
                    .on_toggle(Box::new(move |value| {
                        PluginPreferencesMsg::UpdatePreferenceValue {
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                            id: preference_id.to_owned(),
                            user_data: PluginPreferenceUserDataState::Bool {
                                value: Some(value),
                            },
                        }
                    }))
                    .into();

                let input_field = container(input_field)
                    .padding(Padding::new(8.0))
                    .into();

                vec![input_field]
            }
            PluginPreference::ListOfStrings { .. } => {
                let (value, new_value) = match user_data {
                    None => (None, "".to_owned()),
                    Some(PluginPreferenceUserDataState::ListOfStrings { value, new_value }) => (value.to_owned(), new_value.to_owned()),
                    Some(_) => unreachable!()
                };

                let mut items: Vec<_> = value.clone()
                    .unwrap_or(vec![])
                    .iter()
                    .enumerate()
                    .map(|(index, value_item)| {

                        let mut value = value.clone();
                        if let Some(value) = &mut value {
                            value.remove(index);
                        }

                        let item_text: Element<_> = text_input("", value_item)
                            .width(Length::Fill)
                            .padding(Padding::new(4.0))
                            .into();

                        let remove_icon = text(icons::Bootstrap::Dash)
                            .font(icons::BOOTSTRAP_FONT);

                        let remove_button: Element<_> = button(remove_icon)
                            .style(ButtonStyle::Primary)
                            .on_press(PluginPreferencesMsg::UpdatePreferenceValue {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                                id: preference_id.to_owned(),
                                user_data: PluginPreferenceUserDataState::ListOfStrings {
                                    value,
                                    new_value: new_value.clone(),
                                },
                            })
                            .padding(Padding::from([5.0, 7.0]))
                            .into();

                        let remove_button = container(remove_button)
                            .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                            .into();

                        let item: Element<_> = row([item_text, remove_button])
                            .into();

                        let item = container(item)
                            .padding(Padding::from([4.0, 8.0]))
                            .into();

                        item
                    })
                    .collect();


                let save_value = match &value {
                    None => vec![new_value.clone()],
                    Some(value) => {
                        let mut save_value = value.clone();
                        save_value.push(new_value.clone());
                        save_value
                    }
                };

                let add_msg = if new_value.is_empty() {
                    None
                } else {
                    Some(PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::ListOfStrings {
                            value: Some(save_value),
                            new_value: "".to_owned(),
                        },
                    })
                };

                let add_icon: Element<_> = text(icons::Bootstrap::Plus)
                    .font(icons::BOOTSTRAP_FONT)
                    .into();

                let add_button: Element<_> = button(add_icon)
                    .style(ButtonStyle::Primary)
                    .on_press_maybe(add_msg)
                    .padding(Padding::from([5.0, 7.0]))
                    .into();

                let add_button: Element<_> = container(add_button)
                    .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                    .into();

                let add_text_input: Element<_> = text_input("Enter value...", &new_value)
                    .on_input(move |new_value| PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::ListOfStrings {
                            value: value.clone(),
                            new_value,
                        },
                    })
                    .into();

                let add_item: Element<_> = row([add_text_input, add_button])
                    .into();

                let add_item: Element<_> = container(add_item)
                    .padding(Padding::new(8.0))
                    .into();

                items.push(add_item);

                items
            }
            PluginPreference::ListOfNumbers { .. } => {
                let (value, new_value) = match user_data {
                    None => (None, 0.0),
                    Some(PluginPreferenceUserDataState::ListOfNumbers { value, new_value }) => (value.to_owned(), new_value.to_owned()),
                    Some(_) => unreachable!()
                };


                let mut items: Vec<_> = value.clone()
                    .unwrap_or(vec![])
                    .iter()
                    .enumerate()
                    .map(|(index, value_item)| {

                        let mut value = value.clone();
                        if let Some(value) = &mut value {
                            value.remove(index);
                        }

                        let item_text = text_input("", &format!("{}", value_item))
                            .width(Length::Fill)
                            .padding(Padding::new(4.0))
                            .into();

                        let remove_icon = text(icons::Bootstrap::Dash)
                            .font(icons::BOOTSTRAP_FONT);

                        let remove_button: Element<_> = button(remove_icon)
                            .style(ButtonStyle::Primary)
                            .on_press(PluginPreferencesMsg::UpdatePreferenceValue {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                                id: preference_id.to_owned(),
                                user_data: PluginPreferenceUserDataState::ListOfNumbers {
                                    value,
                                    new_value: new_value.clone(),
                                },
                            })
                            .padding(Padding::from([5.0, 7.0]))
                            .into();

                        let remove_button = container(remove_button)
                            .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                            .into();

                        let item: Element<_> = row([item_text, remove_button])
                            .into();

                        let item = container(item)
                            .padding(Padding::from([4.0, 8.0]))
                            .into();

                        item
                    })
                    .collect();


                let save_value = match &value {
                    None => vec![new_value.clone()],
                    Some(value) => {
                        let mut save_value = value.clone();
                        save_value.push(new_value.clone());
                        save_value
                    }
                };

                let add_icon: Element<_> = text(icons::Bootstrap::Plus)
                    .font(icons::BOOTSTRAP_FONT)
                    .into();

                let add_button: Element<_> = button(add_icon)
                    .style(ButtonStyle::Primary)
                    .on_press(PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::ListOfNumbers {
                            value: Some(save_value),
                            new_value: 0.0,
                        },
                    })
                    .padding(Padding::from([5.0, 7.0]))
                    .into();

                let add_button: Element<_> = container(add_button)
                    .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                    .into();

                let add_number_input: Element<_> = number_input(new_value, f64::MAX, std::convert::identity)
                    .bounds((f64::MIN, f64::MAX))
                    .width(Length::Fill)
                    .into();

                let add_number_input = add_number_input.map(Box::new(move |new_value| {
                    PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::ListOfNumbers {
                            value: value.clone(),
                            new_value,
                        },
                    }
                }));

                let add_item: Element<_> = row([add_number_input, add_button])
                    .into();

                let add_item: Element<_> = container(add_item)
                    .padding(Padding::new(8.0))
                    .into();

                items.push(add_item);

                items
            }
            PluginPreference::ListOfEnums { enum_values, .. } => {
                let (value, new_value) = match user_data {
                    None => (None, None),
                    Some(PluginPreferenceUserDataState::ListOfEnums { value, new_value }) => (value.to_owned(), new_value.to_owned()),
                    Some(_) => unreachable!()
                };

                let mut items: Vec<_> = value.clone()
                    .unwrap_or(vec![])
                    .iter()
                    .enumerate()
                    .map(|(index, value_item)| {

                        let mut value = value.clone();
                        if let Some(value) = &mut value {
                            value.remove(index);
                        }

                        let item_text: Element<_> = text_input("", value_item)
                            .width(Length::Fill)
                            .padding(Padding::new(4.0))
                            .into();

                        let remove_icon = text(icons::Bootstrap::Dash)
                            .font(icons::BOOTSTRAP_FONT);

                        let remove_button: Element<_> = button(remove_icon)
                            .style(ButtonStyle::Primary)
                            .on_press(PluginPreferencesMsg::UpdatePreferenceValue {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                                id: preference_id.to_owned(),
                                user_data: PluginPreferenceUserDataState::ListOfEnums {
                                    value,
                                    new_value: new_value.clone(),
                                },
                            })
                            .padding(Padding::from([5.0, 7.0]))
                            .into();

                        let remove_button = container(remove_button)
                            .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                            .into();

                        let item: Element<_> = row([item_text, remove_button])
                            .into();

                        let item = container(item)
                            .padding(Padding::from([4.0, 8.0]))
                            .into();

                        item
                    })
                    .collect();


                let add_msg = match &new_value {
                    None => None,
                    Some(new_value) => {
                        let save_value = match &value {
                            None => vec![new_value.value.clone()],
                            Some(value) => {
                                let mut save_value = value.clone();
                                save_value.push(new_value.value.clone());
                                save_value
                            }
                        };

                        Some(PluginPreferencesMsg::UpdatePreferenceValue {
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                            id: preference_id.to_owned(),
                            user_data: PluginPreferenceUserDataState::ListOfEnums {
                                value: Some(save_value),
                                new_value: None,
                            },
                        })
                    }
                };


                let add_icon: Element<_> = text(icons::Bootstrap::Plus)
                    .font(icons::BOOTSTRAP_FONT)
                    .into();

                let add_button: Element<_> = button(add_icon)
                    .style(ButtonStyle::Primary)
                    .on_press_maybe(add_msg)
                    .padding(Padding::from([5.0, 7.0]))
                    .into();

                let add_button: Element<_> = container(add_button)
                    .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                    .into();

                let enum_values: Vec<_> = enum_values.iter()
                    .map(|enum_item| SelectItem { label: enum_item.label.to_owned(), value: enum_item.value.to_owned() })
                    .collect();

                let add_enum_input: Element<_> = pick_list(
                    enum_values,
                    new_value,
                    Box::new(move |new_value: SelectItem| PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        id: preference_id.to_owned(),
                        user_data: PluginPreferenceUserDataState::ListOfEnums {
                            value: value.clone(),
                            new_value: Some(new_value),
                        },
                    }),
                )
                    .placeholder("Select value...")
                    .width(Length::Fill)
                    .into();

                let add_item: Element<_> = row([add_enum_input, add_button])
                    .into();

                let add_item: Element<_> = container(add_item)
                    .padding(Padding::new(8.0))
                    .into();

                items.push(add_item);

                items
            }
        };

        for input_field in input_fields {
            column_content.push(input_field);
        }
    }

    column(column_content)
        .into() // todo width full?
}