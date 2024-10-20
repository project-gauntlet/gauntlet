use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;
use anyhow::{anyhow, Context};
use deno_core::{op, OpState, serde_v8, v8};
use deno_core::futures::executor::block_on;
use deno_core::v8::{GetPropertyNamesArgs, KeyConversionMode, PropertyFilter};
use indexmap::IndexMap;
use serde::Deserialize;
use common::model::{EntrypointId, PhysicalKey, UiPropertyValue, UiWidget};
use component_model::{Component, Property, PropertyType, SharedType};
use crate::model::{JsUiRenderLocation, JsUiRequestData, JsUiResponseData, JsUiWidget};
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
fn op_react_replace_view(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    render_location: JsUiRenderLocation,
    top_level_view: bool,
    entrypoint_id: &str,
    container: JsUiWidget,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view...");

    let comp_state = state.borrow();
    let component_model = comp_state.borrow::<ComponentModel>();
    let entrypoint_names = comp_state.borrow::<PluginData>();

    let entrypoint_id = EntrypointId::from_string(entrypoint_id);

    let entrypoint_name = entrypoint_names.entrypoint_names
        .get(&entrypoint_id)
        .expect("entrypoint name for id should always exist")
        .to_string();

    let Component::Root { shared_types, .. } = component_model.components.get("gauntlet:root").unwrap() else {
        unreachable!()
    };

    let data = JsUiRequestData::ReplaceView {
        entrypoint_id,
        entrypoint_name,
        render_location,
        top_level_view,
        container: from_js_to_intermediate_widget(state.clone(), scope, container, component_model, shared_types)?,
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

fn from_js_to_intermediate_widget(state: Rc<RefCell<OpState>>, scope: &mut v8::HandleScope, ui_widget: JsUiWidget, component_model: &ComponentModel, shared_types: &IndexMap<String, SharedType>) -> anyhow::Result<UiWidget> {
    let children = ui_widget.widget_children.into_iter()
        .map(|child| from_js_to_intermediate_widget(state.clone(), scope, child, component_model, shared_types))
        .collect::<anyhow::Result<Vec<UiWidget>>>()?;

    let component = component_model.components
        .get(&ui_widget.widget_type)
        .expect(&format!("component with type {} doesn't exist", &ui_widget.widget_type));

    let empty = vec![];
    let text_part = vec![Property { name: "value".to_owned(), optional: false, property_type: PropertyType::String, description: "".to_string() }];
    let props = match component {
        Component::Standard { props, .. } => props,
        Component::Root { .. } => &empty,
        Component::TextPart { .. } => &text_part,
    };

    let props = props.into_iter()
        .map(|prop| (&prop.name, &prop.property_type))
        .collect::<HashMap<_, _>>();

    let properties = from_js_to_intermediate_properties(state.clone(), scope, ui_widget.widget_properties, &props, shared_types);

    Ok(UiWidget {
        widget_id: ui_widget.widget_id,
        widget_type: ui_widget.widget_type,
        widget_properties: properties?,
        widget_children: children,
    })
}

fn from_js_to_intermediate_properties(
    state: Rc<RefCell<OpState>>,
    scope: &mut v8::HandleScope,
    v8_properties: HashMap<String, serde_v8::Value>,
    component_props: &HashMap<&String, &PropertyType>,
    shared_types: &IndexMap<String, SharedType>
) -> anyhow::Result<HashMap<String, UiPropertyValue>> {
    let vec = v8_properties.into_iter()
        .filter(|(name, _)| name.as_str() != "children")
        .filter(|(_, value)| !value.v8_value.is_function())
        .map(|(name, value)| {
            let val = value.v8_value;

            let Some(property_type) = component_props.get(&name) else {
                return Err(anyhow!("unknown property encountered {:?}", name))
            };

            if !property_type.is_in_property() {
                return Err(anyhow!("unknown property encountered {:?}", name))
            }

            convert(state.clone(), scope, property_type, name, val, shared_types)
        })
        .collect::<anyhow::Result<Vec<(_, _)>>>()?;

    Ok(vec.into_iter().collect())
}

fn convert(
    state: Rc<RefCell<OpState>>,
    scope: &mut v8::HandleScope,
    property_type: &PropertyType,
    name: String,
    value: v8::Local<v8::Value>,
    shared_types: &IndexMap<String, SharedType>
) -> anyhow::Result<(String, UiPropertyValue)> {
    match property_type {
        PropertyType::String | PropertyType::Enum { .. } => {
            if value.is_string() {
                convert_string(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Number => {
            if value.is_number() {
                convert_num(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Boolean => {
            if value.is_boolean() {
                convert_boolean(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Component { .. } => {
            panic!("components should not be present here")
        }
        PropertyType::Function { .. } => {
            panic!("functions are filtered out")
        }
        PropertyType::ImageSource => {
            let source: ImageSource = serde_v8::from_v8(scope, value)?;
            convert_image_source(state.clone(), name, source)
        }
        PropertyType::Object { name: object_name } => {
            if value.is_object() {
                convert_object(state.clone(), scope, name, value, object_name, shared_types)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Union { items } => {
            if value.is_string() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::String | PropertyType::Enum { .. })) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_string(scope, name, value)
                }
            } else if value.is_number() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::Number)) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_num(scope, name, value)
                }
            } else if value.is_boolean() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::Boolean)) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_boolean(scope, name, value)
                }
            } else {
                if !value.is_object() {
                    invalid_type_err(name, value, property_type)
                } else {
                    let image_source = items.iter().find(|prop_type| matches!(prop_type, PropertyType::ImageSource));
                    let object = items.iter().find(|prop_type| matches!(prop_type, PropertyType::Object { .. }));

                    match (image_source, object) {
                        (Some(PropertyType::ImageSource), Some(PropertyType::Object { .. })) => {
                            panic!("image_source and object is conflicting prop_types")
                        }
                        (None, Some(PropertyType::Object { name: object_name })) => {
                            convert_object(state.clone(), scope, name, value, &object_name, shared_types)
                        }
                        (Some(PropertyType::ImageSource), None) => {
                            let source: ImageSource = serde_v8::from_v8(scope, value)?;
                            convert_image_source(state.clone(), name, source)
                        }
                        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => {
                            unreachable!()
                        }
                        (None, None) => {
                            invalid_type_err(name, value, property_type)
                        }
                    }
                }
            }
        }
        PropertyType::Array { .. } => {
            unimplemented!()
        }
    }
}

fn convert_num(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::Number(value.number_value(scope).expect("expected number"))))
}

fn convert_string(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::String(value.to_rust_string_lossy(scope))))
}

fn convert_boolean(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::Bool(value.boolean_value(scope))))
}

fn convert_object(
    state: Rc<RefCell<OpState>>,
    scope: &mut v8::HandleScope,
    name: String,
    value: v8::Local<v8::Value>,
    object_name: &str,
    shared_types: &IndexMap<String, SharedType>
) -> anyhow::Result<(String, UiPropertyValue)> {
    let object: v8::Local<v8::Object> = value.try_into().context(format!("error while reading property {}", name))?;

    let props = object
        .get_own_property_names(scope, GetPropertyNamesArgs {
            property_filter: PropertyFilter::ONLY_ENUMERABLE | PropertyFilter::SKIP_SYMBOLS,
            key_conversion: KeyConversionMode::NoNumbers,
            ..Default::default()
        })
        .context("error getting get_own_property_names".to_string())?;

    let mut result_obj: HashMap<String, UiPropertyValue> = HashMap::new();

    for index in 0..props.length() {
        let key = props.get_index(scope, index).unwrap();
        let value = object.get(scope, key).unwrap();
        let key = key.to_string(scope).unwrap().to_rust_string_lossy(scope);

        let property_type = match shared_types.get(object_name).unwrap() {
            SharedType::Enum { .. } => unreachable!(),
            SharedType::Object { items } => items.get(&key).unwrap()
        };

        let (key, value) = convert(state.clone(), scope, property_type, key, value, shared_types)?;

        result_obj.insert(key, value);
    }

    Ok((name, UiPropertyValue::Object(result_obj)))
}

fn invalid_type_err<T>(name: String, value: v8::Local<v8::Value>, property_type: &PropertyType) -> anyhow::Result<T> {
    Err(anyhow!("invalid type for property {:?}, found: {:?}, expected: {}", name, value.type_repr(), expected_type(property_type)))
}

fn expected_type(prop_type: &PropertyType) -> String {
    match prop_type {
        PropertyType::String => "string".to_owned(),
        PropertyType::Number => "number".to_owned(),
        PropertyType::Boolean => "boolean".to_owned(),
        PropertyType::Component { .. } => {
            panic!("components should not be present here")
        }
        PropertyType::Function { .. } => {
            panic!("functions are filtered out")
        }
        PropertyType::ImageSource => "ImageSource".to_owned(),
        PropertyType::Enum { .. } => "enum".to_owned(),
        PropertyType::Union { items } => {
            items.into_iter()
                .map(|prop_type| expected_type(prop_type))
                .collect::<Vec<_>>()
                .join(", ")
        },
        PropertyType::Object { .. } => "object".to_owned(),
        PropertyType::Array { .. } => {
            unimplemented!()
        }
    }
}

fn convert_image_source(state: Rc<RefCell<OpState>>, name: String, source: ImageSource) -> anyhow::Result<(String, UiPropertyValue)> {
    match source {
        ImageSource::Asset { asset } => {
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
        ImageSource::Url { url } => {
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


#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ImageSource {
    Asset {
        asset: String
    },
    Url {
        url: String
    }
}