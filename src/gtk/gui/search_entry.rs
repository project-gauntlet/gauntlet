use std::path::PathBuf;

use gtk::prelude::*;
use relm4::typed_list_view::RelmListItem;

#[derive(Debug, Clone)]
pub struct SearchEntry {
    entrypoint_name: String,
    entrypoint_id: String,
    plugin_name: String,
    plugin_id: String,
    image_path: Option<PathBuf>,
}

impl SearchEntry {
    pub(crate) fn new(
        entrypoint_name: &str,
        entrypoint_id: &str,
        plugin_name: &str,
        plugin_id: &str,
        image_path: Option<PathBuf>,
    ) -> Self {
        Self {
            entrypoint_name: entrypoint_name.to_owned(),
            entrypoint_id: entrypoint_id.to_owned(),
            plugin_name: plugin_name.to_owned(),
            plugin_id: plugin_id.to_owned(),
            image_path,
        }
    }

    pub fn entrypoint_name(&self) -> &str {
        &self.entrypoint_name
    }

    pub fn entrypoint_id(&self) -> &str {
        &self.entrypoint_id
    }

    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn image_path(&self) -> &Option<PathBuf> {
        &self.image_path
    }
}

pub struct Widgets {
    image: gtk::Image,
    label: gtk::Label,
    sub_label: gtk::Label,
}

impl RelmListItem for SearchEntry {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        relm4::view! {
            my_box = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_top: 6,
                set_margin_bottom: 6,
                set_margin_start: 6,
                set_margin_end: 6,

                #[name = "image"]
                gtk::Image,

                #[name = "label"]
                gtk::Label {
                    set_margin_start: 6,
                },

                #[name = "sub_label"]
                gtk::Label {
                    set_margin_start: 6,
                },
            }
        }

        let widgets = Widgets {
            image,
            label,
            sub_label,
        };

        (my_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets {
            image,
            label,
            sub_label,
        } = widgets;

        if let Some(path) = &self.image_path {
            image.set_file(Some(path.to_str().unwrap())) // FIXME this shouldn't be fallible
        }

        label.set_label(&self.entrypoint_name);
        sub_label.set_label(&self.plugin_name);
    }
}