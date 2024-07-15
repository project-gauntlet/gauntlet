use iced::{Alignment, Command, Length};
use iced::alignment::Horizontal;
use iced::widget::{column, container, row, Space, text};

use common::model::{PhysicalKey, PhysicalShortcut};
use common::rpc::backend_api::BackendApi;

use crate::components::shortcut_selector::ShortcutSelector;
use crate::theme::Element;
use crate::theme::shortcut_selector::ShortcutSelectorStyle;

pub struct ManagementAppGeneralState {
    backend_api: Option<BackendApi>,
    current_shortcut: PhysicalShortcut
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsgIn {
    ShortcutCaptured(PhysicalShortcut),
    SetShortcut(PhysicalShortcut),
    Noop
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsgOut {
    Noop
}

impl ManagementAppGeneralState {
    pub fn new(backend_api: Option<BackendApi>) -> Self {
        let shortcut = PhysicalShortcut {
            physical_key: PhysicalKey::Space,
            modifier_shift: false,
            modifier_control: false,
            modifier_alt: false,
            modifier_meta: true,
        };

        Self {
            backend_api,
            current_shortcut: shortcut
        }
    }

    pub fn update(&mut self, message: ManagementAppGeneralMsgIn) -> Command<ManagementAppGeneralMsgOut> {
        let backend_api = match &self.backend_api {
            Some(backend_api) => backend_api.clone(),
            None => {
                return Command::none()
            }
        };

        match message {
            ManagementAppGeneralMsgIn::ShortcutCaptured(shortcut) => {
                self.current_shortcut = shortcut.clone();

                let mut backend_api = backend_api.clone();

                Command::perform(async move {
                    backend_api.set_global_shortcut(shortcut)
                        .await
                        .unwrap() // TODO proper error handling
                }, |_| ManagementAppGeneralMsgOut::Noop)
            }
            ManagementAppGeneralMsgIn::Noop => {
                Command::none()
            }
            ManagementAppGeneralMsgIn::SetShortcut(shortcut) => {
                self.current_shortcut = shortcut.clone();

                Command::perform(async move {}, |_| ManagementAppGeneralMsgOut::Noop)
            }
        }
    }

    pub fn view(&self) -> Element<ManagementAppGeneralMsgIn> {
        let on_shortcut_captured = Box::new(move |value| {
            ManagementAppGeneralMsgIn::ShortcutCaptured(value)
        });

        let shortcut_selector: Element<_> = ShortcutSelector::new(
            &self.current_shortcut,
            on_shortcut_captured,
            ShortcutSelectorStyle::Default
        ).into();

        let field: Element<_> = container(shortcut_selector)
            .width(Length::Fill)
            .height(Length::Fixed(35.0))
            .into();

        let field = self.view_field("Global Shortcut", field.into());

        let content: Element<_> = column(vec![field])
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .into();

        content
    }

    fn view_field<'a>(&self, label: &str, input: Element<'a, ManagementAppGeneralMsgIn>) -> Element<'a, ManagementAppGeneralMsgIn> {
        let label: Element<_> = text(label)
            .horizontal_alignment(Horizontal::Right)
            .width(Length::Fill)
            .into();

        let label: Element<_> = container(label)
            .width(Length::FillPortion(3))
            .padding(4)
            .into();

        let input_field = container(input)
            .width(Length::FillPortion(3))
            .padding(4)
            .into();

        let after = Space::with_width(Length::FillPortion(2))
            .into();

        let content = vec![
            label,
            input_field,
            after,
        ];

        let row: Element<_> = row(content)
            .align_items(Alignment::Center)
            .padding(12)
            .into();

        row
    }
}
