use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;
use anyhow::{anyhow, Context};
use deno_core::{op2, OpState, serde_v8, v8};
use deno_core::futures::executor::block_on;
use deno_core::v8::{GetPropertyNamesArgs, KeyConversionMode, PropertyFilter};
use indexmap::IndexMap;
use serde::{de, Deserialize, Deserializer};
use serde::de::Error;
use common::model::{ActionPanelSectionWidget, ActionPanelSectionWidgetOrderedMembers, ActionPanelWidget, ActionPanelWidgetOrderedMembers, ActionWidget, CheckboxWidget, CodeBlockWidget, ContentWidget, ContentWidgetOrderedMembers, DatePickerWidget, DetailWidget, EmptyViewWidget, EntrypointId, FormWidget, FormWidgetOrderedMembers, GridItemWidget, GridSectionWidget, GridSectionWidgetOrderedMembers, GridWidget, GridWidgetOrderedMembers, H1Widget, H2Widget, H3Widget, H4Widget, H5Widget, H6Widget, HorizontalBreakWidget, IconAccessoryWidget, Image, ImageSource, ImageSourceAsset, ImageSourceUrl, ImageWidget, InlineSeparatorWidget, InlineWidget, InlineWidgetOrderedMembers, ListItemAccessories, ListItemWidget, ListSectionWidget, ListSectionWidgetOrderedMembers, ListWidget, ListWidgetOrderedMembers, MetadataIconWidget, MetadataLinkWidget, MetadataSeparatorWidget, MetadataTagItemWidget, MetadataTagListWidget, MetadataTagListWidgetOrderedMembers, MetadataValueWidget, MetadataWidget, MetadataWidgetOrderedMembers, ParagraphWidget, PasswordFieldWidget, PhysicalKey, PluginId, RootWidget, RootWidgetMembers, SearchBarWidget, SelectItemWidget, SelectWidget, SelectWidgetOrderedMembers, SeparatorWidget, TextAccessoryWidget, TextFieldWidget, UiPropertyValue, UiRenderLocation, UiWidgetId, WidgetVisitor};
use common_plugin_runtime::backend_for_plugin_runtime_api::BackendForPluginRuntimeApi;
use component_model::{Component, Property, PropertyType, SharedType};
use component_model::Component::Root;
use crate::model::JsUiRenderLocation;
use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::js::{ComponentModel, PluginData, BackendForPluginRuntimeApiImpl};

#[op2]
pub fn show_plugin_error_view(state: Rc<RefCell<OpState>>, #[string] entrypoint_id: String, #[serde] render_location: JsUiRenderLocation) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    let render_location = match render_location {
        JsUiRenderLocation::InlineView => UiRenderLocation::InlineView,
        JsUiRenderLocation::View => UiRenderLocation::View,
    };

    block_on(async {
        api.ui_show_plugin_error_view(
            EntrypointId::from_string(entrypoint_id),
            render_location,
        ).await
    })
}

#[op2(fast)]
pub fn show_preferences_required_view(state: Rc<RefCell<OpState>>, #[string] entrypoint_id: String, plugin_preferences_required: bool, entrypoint_preferences_required: bool) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    block_on(async {
        api.ui_show_preferences_required_view(
            EntrypointId::from_string(entrypoint_id),
            plugin_preferences_required,
            entrypoint_preferences_required
        ).await
    })
}

#[op2(fast)]
pub fn clear_inline_view(state: Rc<RefCell<OpState>>) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    block_on(async { api.ui_clear_inline_view().await })
}

#[op2]
#[string]
pub fn op_inline_view_endpoint_id(state: Rc<RefCell<OpState>>) -> Option<String> {
    state.borrow()
        .borrow::<PluginData>()
        .inline_view_entrypoint_id()
        .clone()
}

#[op2]
pub fn op_react_replace_view<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    #[serde] render_location: JsUiRenderLocation,
    top_level_view: bool,
    #[string] entrypoint_id: &str,
    #[serde] container: serde_v8::Value<'a>,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view...");

    let mut deserializer = serde_v8::Deserializer::new(scope, container.v8_value, None);

    #[cfg(feature = "scenario_runner")]
    let container_value = serde_value::Value::deserialize(&mut deserializer)?;
    let container = RootWidget::deserialize(&mut deserializer)?;

    let images = ImageGatherer::run_gatherer(state.clone(), &container)?;

    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    let render_location = match render_location {
        JsUiRenderLocation::InlineView => UiRenderLocation::InlineView,
        JsUiRenderLocation::View => UiRenderLocation::View,
    };

    block_on(async {
        api.ui_render(
            EntrypointId::from_string(entrypoint_id),
            render_location,
            top_level_view,
            container,
            #[cfg(feature = "scenario_runner")]
            container_value,
            images
        ).await
    })
}

#[op2]
#[serde]
pub fn op_component_model(state: Rc<RefCell<OpState>>) -> HashMap<String, Component> {
    state.borrow()
        .borrow::<ComponentModel>()
        .components
        .clone()
}

#[op2(async)]
#[string]
pub async fn fetch_action_id_for_shortcut(
    state: Rc<RefCell<OpState>>,
    #[string] entrypoint_id: String,
    #[string] key: String,
    modifier_shift: bool,
    modifier_control: bool,
    modifier_alt: bool,
    modifier_meta: bool
) -> anyhow::Result<Option<String>> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    let result = api.ui_get_action_id_for_shortcut(
        EntrypointId::from_string(entrypoint_id),
        key,
        modifier_shift,
        modifier_control,
        modifier_alt,
        modifier_meta
    ).await?;

    Ok(result)
}

#[op2(async)]
pub async fn show_hud(state: Rc<RefCell<OpState>>, #[string] display: String) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    block_on(async { api.ui_show_hud(display).await })
}

#[op2(async)]
pub async fn update_loading_bar(state: Rc<RefCell<OpState>>, #[string] entrypoint_id: String, show: bool) -> anyhow::Result<()> {
    let api = {
        let state = state.borrow();

        let api = state
            .borrow::<BackendForPluginRuntimeApiImpl>()
            .clone();

        api
    };

    block_on(async { api.ui_update_loading_bar(EntrypointId::from_string(entrypoint_id), show).await })
}

struct ImageGatherer {
    state: Rc<RefCell<OpState>>,
    image_sources: HashMap<UiWidgetId, anyhow::Result<bytes::Bytes>>
}

impl WidgetVisitor for ImageGatherer {
    fn image(&mut self, widget_id: UiWidgetId, widget: &Image) {
        if let Image::ImageSource(image_source) = &widget {
            self.image_sources.insert(widget_id, get_image_date(self.state.clone(), image_source));
        }
    }
}

impl ImageGatherer {
    fn run_gatherer(state: Rc<RefCell<OpState>>, root_widget: &RootWidget) -> anyhow::Result<HashMap<UiWidgetId, bytes::Bytes>> {
        let mut gatherer = Self {
            state,
            image_sources: HashMap::new()
        };

        gatherer.root_widget(root_widget);

        gatherer.image_sources
            .into_iter()
            .map(|(widget_id, image)| image.map(|image| (widget_id, image)))
            .collect::<anyhow::Result<_>>()
    }
}

fn get_image_date(state: Rc<RefCell<OpState>>, source: &ImageSource) -> anyhow::Result<bytes::Bytes> {
    match source {
        ImageSource::ImageSourceAsset(ImageSourceAsset { asset }) => {
            let bytes = {
                let state = state.borrow();

                let api = state
                    .borrow::<BackendForPluginRuntimeApiImpl>()
                    .clone();

                block_on(async {
                    api.get_asset_data(&asset).await
                })?
            };

            Ok(bytes::Bytes::from(bytes))
        }
        ImageSource::ImageSourceUrl(ImageSourceUrl { url }) => {
            // FIXME implement error handling so it doesn't error whole view
            // TODO implement caching

            let bytes: bytes::Bytes = ureq::get(&url)
                .call()?
                .into_reader()
                .bytes()
                .collect::<std::io::Result<Vec<u8>>>()?
                .into();

            Ok(bytes)
        }
    }
}

#[allow(unused)]
fn debug_object_to_json(
    scope: &mut v8::HandleScope,
    val: v8::Local<v8::Value>
) -> String {
    let local = scope.get_current_context();
    let global = local.global(scope);
    let json_string = v8::String::new(scope, "Deno").expect("Unable to create Deno string");
    let json_object = global.get(scope, json_string.into()).expect("Global Deno object not found");
    let json_object: v8::Local<v8::Object> = json_object.try_into().expect("Deno value is not an object");
    let inspect_string = v8::String::new(scope, "inspect").expect("Unable to create inspect string");
    let inspect_object = json_object.get(scope, inspect_string.into()).expect("Unable to get inspect on global Deno object");
    let stringify_fn: v8::Local<v8::Function> = inspect_object.try_into().expect("inspect value is not a function");;
    let undefined = v8::undefined(scope).into();

    let json_object = stringify_fn.call(scope, undefined, &[val]).expect("Unable to get serialize prop");
    let json_string: v8::Local<v8::String> = json_object.try_into().expect("result is not a string");

    let result = json_string.to_rust_string_lossy(scope);

    result
}
