use std::collections::HashMap;

use gauntlet_common::model::IconAccessoryWidget;
use gauntlet_common::model::ImageLike;
use gauntlet_common::model::SearchResult;
use gauntlet_common::model::SearchResultAccessory;
use gauntlet_common::model::SearchResultEntrypointType;
use gauntlet_common::model::TextAccessoryWidget;
use iced::Alignment;
use iced::Length;
use iced::advanced::image::Handle;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text::Shaping;

use crate::ui::scroll_handle::ScrollContent;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::image::ImageStyle;
use crate::ui::theme::space::ThemeKindSpace;
use crate::ui::theme::text::TextStyle;
use crate::ui::widget::accessories::render_icon_accessory;
use crate::ui::widget::accessories::render_text_accessory;

pub fn search_list<'a>(
    search_results: &'a ScrollContent<SearchResult>,
    focused_search_result: &ScrollHandle,
) -> Element<'a, SearchResult> {
    let items: Vec<Element<_>> = search_results
        .items()
        .iter()
        .map(|(index, search_result)| {
            let entrypoint_name: Element<_> = text(&search_result.entrypoint_name)
                .size(15)
                .shaping(Shaping::Advanced)
                .into();
            let entrypoint_name: Element<_> = container(entrypoint_name).themed(ContainerStyle::MainListItemText);

            let spacer: Element<_> = horizontal_space().width(Length::Fill).into();

            let plugin_name_text: Element<_> = text(search_result.plugin_name.clone())
                .size(15)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::MainListItemSubtext);

            let plugin_name_text: Element<_> = container(plugin_name_text).themed(ContainerStyle::MainListItemSubText); // FIXME find a way to set padding based on whether the scroll bar is visible

            let mut button_content = vec![];

            if let Some(path) = &search_result.entrypoint_icon {
                let image: Element<_> =
                    iced::widget::image(Handle::from_bytes(path.clone())).themed(ImageStyle::MainListItemIcon);

                let image: Element<_> = container(image).themed(ContainerStyle::MainListItemIcon);

                button_content.push(image);
            } else {
                let spacer: Element<_> = horizontal_space() // TODO replace with grayed out gauntlet icon
                    .themed(ThemeKindSpace::MainListItemIcon);

                let spacer: Element<_> = container(spacer).themed(ContainerStyle::MainListItemIcon);

                button_content.push(spacer);
            }

            button_content.push(entrypoint_name);
            button_content.push(plugin_name_text);

            if let Some(alias) = &search_result.entrypoint_alias {
                let alias: Element<_> = text(alias.clone()).shaping(Shaping::Advanced).size(15).into();

                let alias: Element<_> = container(alias).themed(ContainerStyle::MainListItemAlias).into();

                button_content.push(alias);
            }

            button_content.push(spacer);

            if search_result.entrypoint_accessories.len() > 0 {
                let accessories: Vec<Element<_>> = search_result
                    .entrypoint_accessories
                    .iter()
                    .map(|accessory| {
                        match accessory {
                            SearchResultAccessory::TextAccessory { text, icon, tooltip } => {
                                render_text_accessory(
                                    &HashMap::new(),
                                    &TextAccessoryWidget {
                                        __id__: 0,
                                        text: text.clone(),
                                        icon: icon.as_ref().map(|icon| ImageLike::Icons(icon.clone())),
                                        tooltip: tooltip.clone(),
                                    },
                                )
                            }
                            SearchResultAccessory::IconAccessory { icon, tooltip } => {
                                render_icon_accessory(
                                    &HashMap::new(),
                                    &IconAccessoryWidget {
                                        __id__: 0,
                                        icon: ImageLike::Icons(icon.clone()),
                                        tooltip: tooltip.clone(),
                                    },
                                )
                            }
                        }
                    })
                    .collect();

                let accessories: Element<_> = row(accessories).into();

                button_content.push(accessories);
            }

            let type_text = match search_result.entrypoint_type {
                SearchResultEntrypointType::Command => "Command",
                SearchResultEntrypointType::View => "View",
                SearchResultEntrypointType::Generated => {
                    match &search_result.entrypoint_generator_name {
                        None => "",
                        Some(entrypoint_generator_name) => entrypoint_generator_name,
                    }
                }
            };

            let type_text: Element<_> = text(type_text.to_string())
                .size(15)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::MainListItemSubtext);

            let type_text: Element<_> = container(type_text).themed(ContainerStyle::MainListItemSubText);

            button_content.push(type_text);

            let button_content: Element<_> = row(button_content).align_y(Alignment::Center).into();

            let style = match &focused_search_result.current_item_id {
                None => ButtonStyle::MainListItem,
                Some(focused_index) => {
                    if focused_index == index {
                        ButtonStyle::MainListItemFocused
                    } else {
                        ButtonStyle::MainListItem
                    }
                }
            };

            let content = button(button_content)
                .width(Length::Fill)
                .on_press(search_result.clone())
                .themed(style);

            container(content).id(index.clone()).into()
        })
        .collect();

    column(items).into()
}
