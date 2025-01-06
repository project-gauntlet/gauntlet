use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::image::ImageStyle;
use crate::ui::theme::space::ThemeKindSpace;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::{Element, ThemableWidget};
use crate::ui::widget::{render_icon_accessory, render_text_accessory};
use std::collections::HashMap;

use gauntlet_common::model::{IconAccessoryWidget, ImageLike, SearchResult, SearchResultAccessory, TextAccessoryWidget};
use iced::advanced::image::Handle;
use iced::widget::button;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text::Shaping;
use iced::widget::{column, container, horizontal_space};
use iced::{Alignment, Length};

pub fn search_list<'a>(
    search_results: &'a [SearchResult],
    focused_search_result: &ScrollHandle<SearchResult>,
) -> Element<'a, SearchResult> {
    let items: Vec<Element<_>> = search_results
        .iter()
        .enumerate()
        .map(|(index, search_result)| {
            let main_text: Element<_> = text(&search_result.entrypoint_name)
                .shaping(Shaping::Advanced)
                .into();
            let main_text: Element<_> = container(main_text)
                .themed(ContainerStyle::MainListItemText);

            let spacer: Element<_> = horizontal_space()
                .width(Length::Fill)
                .into();

            let sub_text = match &search_result.entrypoint_generator_name {
                None => &search_result.plugin_name,
                Some(entrypoint_generator_name) => &format!("{} - {}", entrypoint_generator_name, &search_result.plugin_name)
            };

            let sub_text: Element<_> = text(sub_text.clone())
                .shaping(Shaping::Advanced)
                .themed(TextStyle::MainListItemSubtext);

            let sub_text: Element<_> = container(sub_text)
                .themed(ContainerStyle::MainListItemSubText); // FIXME find a way to set padding based on whether the scroll bar is visible

            let mut button_content = vec![];

            if let Some(path) = &search_result.entrypoint_icon {
                let image: Element<_> = iced::widget::image(Handle::from_path(path))
                    .themed(ImageStyle::MainListItemIcon);

                let image: Element<_> = container(image)
                    .themed(ContainerStyle::MainListItemIcon);

                button_content.push(image);
            } else {
                let spacer: Element<_> = horizontal_space() // TODO replace with grayed out gauntlet icon
                        .themed(ThemeKindSpace::MainListItemIcon);

                let spacer: Element<_> = container(spacer)
                    .themed(ContainerStyle::MainListItemIcon);

                button_content.push(spacer);
            }

            button_content.push(main_text);
            button_content.push(spacer);

            if search_result.entrypoint_accessories.len() > 0 {
                let accessories: Vec<Element<_>> = search_result.entrypoint_accessories
                    .iter()
                    .map(|accessory| {
                        match accessory {
                            SearchResultAccessory::TextAccessory { text, icon, tooltip } => {
                                render_text_accessory(&HashMap::new(), &TextAccessoryWidget {
                                    __id__: 0,
                                    text: text.clone(),
                                    icon: icon.as_ref().map(|icon| ImageLike::Icons(icon.clone())),
                                    tooltip: tooltip.clone(),
                                })
                            },
                            SearchResultAccessory::IconAccessory { icon, tooltip } => {
                                render_icon_accessory(&HashMap::new(), &IconAccessoryWidget {
                                    __id__: 0,
                                    icon: ImageLike::Icons(icon.clone()),
                                    tooltip: tooltip.clone(),
                                })
                            }
                        }
                    })
                    .collect();

                let accessories: Element<_> = row(accessories)
                    .into();

                button_content.push(accessories);
            }

            button_content.push(sub_text);

            let button_content: Element<_> = row(button_content)
                .align_y(Alignment::Center)
                .into();

            let style = match focused_search_result.index {
                None => ButtonStyle::MainListItem,
                Some(focused_index) => {
                    if focused_index == index {
                        ButtonStyle::MainListItemFocused
                    } else {
                        ButtonStyle::MainListItem
                    }
                }
            };

            button(button_content)
                .width(Length::Fill)
                .on_press(search_result.clone())
                .themed(style)
        })
        .collect();

    column(items).into()
}
