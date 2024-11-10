use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;
use anyhow::{anyhow, Context};
use deno_core::{op, OpState, serde_v8, v8};
use deno_core::futures::executor::block_on;
use deno_core::v8::{GetPropertyNamesArgs, KeyConversionMode, PropertyFilter};
use indexmap::IndexMap;
use serde::{de, Deserialize, Deserializer};
use serde::de::Error;
use common::model::{EntrypointId, PhysicalKey, RootWidget, UiPropertyValue, UiWidget};
use component_model::{Component, Property, PropertyType, SharedType};
use component_model::Component::Root;
use crate::model::{JsUiRenderLocation, JsUiRequestData, JsUiResponseData};
use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::js::{ComponentModel, make_request, PluginData};

#[op]
fn show_plugin_error_view(state: Rc<RefCell<OpState>>, entrypoint_id: String, render_location: JsUiRenderLocation) -> anyhow::Result<()> {
    let data = JsUiRequestData::ShowPluginErrorView {
        entrypoint_id: EntrypointId::from_string(entrypoint_id),
        render_location,
    };

    match make_request(&state, data).context("ClearInlineView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling show_plugin_error_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn show_preferences_required_view(state: Rc<RefCell<OpState>>, entrypoint_id: String, plugin_preferences_required: bool, entrypoint_preferences_required: bool) -> anyhow::Result<()> {
    let data = JsUiRequestData::ShowPreferenceRequiredView {
        entrypoint_id: EntrypointId::from_string(entrypoint_id),
        plugin_preferences_required,
        entrypoint_preferences_required
    };

    match make_request(&state, data).context("ShowPreferenceRequiredView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling show_preferences_required_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn clear_inline_view(state: Rc<RefCell<OpState>>) -> anyhow::Result<()> {
    let data = JsUiRequestData::ClearInlineView;

    match make_request(&state, data).context("ClearInlineView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling clear_inline_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn op_inline_view_endpoint_id(state: Rc<RefCell<OpState>>) -> Option<String> {
    state.borrow()
        .borrow::<PluginData>()
        .inline_view_entrypoint_id()
        .clone()
}

#[op(v8)]
fn op_react_replace_view<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    render_location: JsUiRenderLocation,
    top_level_view: bool,
    entrypoint_id: &str,
    container: serde_v8::Value<'a>,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view...");

    let comp_state = state.borrow();
    let entrypoint_names = comp_state.borrow::<PluginData>();

    let entrypoint_id = EntrypointId::from_string(entrypoint_id);

    let entrypoint_name = entrypoint_names.entrypoint_names
        .get(&entrypoint_id)
        .expect("entrypoint name for id should always exist")
        .to_string();

    let mut deserializer = serde_v8::Deserializer::new(scope, container.v8_value, None);
    let container = RootWidget::deserialize(&mut deserializer)?;

    let data = JsUiRequestData::ReplaceView {
        entrypoint_id,
        entrypoint_name,
        render_location,
        top_level_view,
        container,
    };

    match make_request(&state, data).context("ReplaceView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn op_component_model(state: Rc<RefCell<OpState>>) -> HashMap<String, Component> {
    state.borrow()
        .borrow::<ComponentModel>()
        .components
        .clone()
}

#[op]
async fn fetch_action_id_for_shortcut(
    state: Rc<RefCell<OpState>>,
    entrypoint_id: String,
    key: String,
    modifier_shift: bool,
    modifier_control: bool,
    modifier_alt: bool,
    modifier_meta: bool
) -> anyhow::Result<Option<String>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let result = repository.get_action_id_for_shortcut(
        &plugin_id.to_string(),
        &entrypoint_id,
        PhysicalKey::from_value(key),
        modifier_shift,
        modifier_control,
        modifier_alt,
        modifier_meta
    ).await?;

    Ok(result)
}

#[op]
async fn show_hud(state: Rc<RefCell<OpState>>, display: String) -> anyhow::Result<()> {
    let data = JsUiRequestData::ShowHud {
        display
    };

    match make_request(&state, data).context("ShowHud frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!("Calling show_hud returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

// TODO move to separate file
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ImageSourceOld {
    Asset {
        asset: String
    },
    Url {
        url: String
    }
}

fn convert_image_source(state: Rc<RefCell<OpState>>, name: String, source: ImageSourceOld) -> anyhow::Result<(String, UiPropertyValue)> {
    match source {
        ImageSourceOld::Asset { asset } => {
            let bytes = {
                let state = state.borrow();

                let plugin_id = state
                    .borrow::<PluginData>()
                    .plugin_id()
                    .clone();

                let repository = state
                    .borrow::<DataDbRepository>()
                    .clone();

                block_on(async {
                    repository.get_asset_data(&plugin_id.to_string(), &asset).await
                })?
            };

            Ok((name, UiPropertyValue::Bytes(bytes::Bytes::from(bytes))))
        }
        ImageSourceOld::Url { url } => {
            // FIXME implement error handling so it doesn't error whole view
            // TODO implement caching

            let bytes: bytes::Bytes = ureq::get(&url)
                .call()?
                .into_reader()
                .bytes()
                .collect::<std::io::Result<Vec<u8>>>()?
                .into();

            Ok((name, UiPropertyValue::Bytes(bytes)))
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
