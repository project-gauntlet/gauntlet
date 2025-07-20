use deno_core::v8;
use gauntlet_common::model::*;

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid type (expected {expected:?}, got {found:?}) at path '{}'", .path.join("."))]
    UnexpectedType {
        expected: &'static str,
        found: &'static str,
        path: Vec<String>,
    },
    #[error("unexpected component (expected one of [{expected:?}], got {found:?}) at path '{}'", .path.join("."))]
    UnexpectedComponent {
        found: String,
        expected: &'static str,
        path: Vec<String>,
    },
    #[error("enum '{enum_name}' does not accept '{value}' as one of it's values, at path '{}'", .path.join("."))]
    InvalidEnumValue {
        enum_name: &'static str,
        value: String,
        path: Vec<String>,
    },
    #[error("required property '{name}' is not provided at path '{}'", .path.join("."))]
    RequiredProp { name: &'static str, path: Vec<String> },
    #[error("only single '{name}' component can be specified at path '{}'", .path.join("."))]
    SingleComponent { name: &'static str, path: Vec<String> },
    #[error("required component '{name}' is not provided at path '{}'", .path.join("."))]
    RequiredComponent { name: &'static str, path: Vec<String> },
    #[error("unknown property(ies) [{}] at path '{}'", .names.join(", "), .path.join("."))]
    UnknownProp { names: Vec<String>, path: Vec<String> },
    #[error("internal: {msg} at path '{}'", .path.join("."))]
    InternalError { msg: String, path: Vec<String> },
    #[error("internal: {msg} at path '{}'", .path.join("."))]
    InternalErrorWithSource {
        msg: String,
        path: Vec<String>,
        #[source]
        source: Box<dyn std::error::Error>,
    },
}

macro_rules! error_internal {
    ($($arg:tt)*) => {
        Error::internal(format!($($arg)*))
    };
}
macro_rules! error_internal_source {
    ($err:expr, $($arg:tt)*) => {
        Error::internal_with_source(format!($($arg)*), Box::new($err))
    };
}

impl Error {
    fn unexpected_type(found: &'static str, expected: &'static str) -> Self {
        Self::UnexpectedType {
            expected,
            found,
            path: vec![],
        }
    }

    fn unexpected_component(found: &str, expected: &'static str) -> Self {
        Self::UnexpectedComponent {
            found: found.to_string(),
            expected,
            path: vec![],
        }
    }

    fn unknown_prop(names: Vec<String>) -> Self {
        Self::UnknownProp { names, path: vec![] }
    }

    fn required_prop(name: &'static str) -> Self {
        Self::RequiredProp { name, path: vec![] }
    }

    fn required_component(name: &'static str) -> Self {
        Self::RequiredComponent { name, path: vec![] }
    }

    fn single_component(name: &'static str) -> Self {
        Self::SingleComponent { name, path: vec![] }
    }

    fn invalid_enum_value(enum_name: &'static str, value: &str) -> Self {
        Self::InvalidEnumValue {
            enum_name,
            value: value.to_string(),
            path: vec![],
        }
    }

    fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError {
            msg: msg.into(),
            path: vec![],
        }
    }

    fn internal_with_source(msg: impl Into<String>, source: Box<dyn std::error::Error>) -> Self {
        Self::InternalErrorWithSource {
            msg: msg.into(),
            path: vec![],
            source,
        }
    }

    fn inside(path_segment: &'static str) -> impl Fn(Error) -> Error {
        fn prepend(path: &mut Vec<String>, path_segment: &'static str) {
            path.insert(0, path_segment.to_string());
        }

        move |mut err: Error| {
            match &mut err {
                Error::UnexpectedType { path, .. } => prepend(path, path_segment),
                Error::InternalError { path, .. } => prepend(path, path_segment),
                Error::InternalErrorWithSource { path, .. } => prepend(path, path_segment),
                Error::UnknownProp { path, .. } => prepend(path, path_segment),
                Error::RequiredProp { path, .. } => prepend(path, path_segment),
                Error::SingleComponent { path, .. } => prepend(path, path_segment),
                Error::RequiredComponent { path, .. } => prepend(path, path_segment),
                Error::UnexpectedComponent { path, .. } => prepend(path, path_segment),
                Error::InvalidEnumValue { path, .. } => prepend(path, path_segment),
            }

            err
        }
    }
}

fn deserialize_widget_type(scope: &mut v8::HandleScope, container: v8::Local<v8::Object>) -> Result<String> {
    let widget_type = extract_object_value(scope, container, "widgetType")
        .ok_or(error_internal!("'widgetType' field is not present on widget object"))?;

    let widget_type: v8::Local<v8::String> = widget_type.try_into().map_err(|_| {
        error_internal!(
            "invalid 'widgetType', expected 'string', got: '{}'",
            widget_type.type_repr()
        )
    })?;

    Ok(widget_type.to_rust_string_lossy(scope))
}

fn deserialize_boolean(value: v8::Local<v8::Value>) -> Result<bool> {
    if value.is_boolean() {
        Ok(value.is_true())
    } else {
        Err(Error::unexpected_type(value.type_repr(), "boolean"))
    }
}

fn deserialize_string(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) -> Result<String> {
    let value: v8::Local<v8::String> = value
        .try_into()
        .map_err(|_| Error::unexpected_type(value.type_repr(), "string"))?;

    Ok(value.to_rust_string_lossy(scope))
}

fn deserialize_number(value: v8::Local<v8::Value>) -> Result<f64> {
    let value: v8::Local<v8::Number> = value
        .try_into()
        .map_err(|_| Error::unexpected_type(value.type_repr(), "number"))?;

    Ok(value.value())
}

fn deserialize_text_widget(scope: &mut v8::HandleScope, container: v8::Local<v8::Object>) -> Result<String> {
    let value = extract_object_value(scope, container, "widgetProperties")
        .ok_or_else(|| error_internal!("'widgetProperties' field not present on widget object"))?;

    let value: v8::Local<v8::Object> = value.try_into().map_err(|_| {
        error_internal!(
            "invalid 'widgetProperties', expected 'object', got: '{}'",
            value.type_repr()
        )
    })?;

    let value = extract_object_value(scope, value, "value")
        .ok_or_else(|| error_internal!("'value' field not present on widget object"))?;

    let value: v8::Local<v8::String> = value
        .try_into()
        .map_err(|_| error_internal!("invalid 'value', expected 'string', got: '{}'", value.type_repr()))?;

    Ok(value.to_rust_string_lossy(scope))
}

fn extract_object_value<'a, 'b: 'a>(
    scope: &mut v8::HandleScope<'b>,
    object: v8::Local<v8::Object>,
    key: &str,
) -> Option<v8::Local<'a, v8::Value>> {
    let key = v8::String::new(scope, key).unwrap();
    let value = object.get(scope, key.into());
    value.filter(|value| !value.is_null_or_undefined())
}

fn extract_object_keys(scope: &mut v8::HandleScope, object: v8::Local<v8::Object>) -> Result<Vec<String>> {
    let names_args = v8::GetPropertyNamesArgsBuilder::new()
        .key_conversion(v8::KeyConversionMode::ConvertToString)
        .build();

    let keys = match object.get_own_property_names(scope, names_args) {
        Some(keys) => {
            let mut result = vec![];

            for index in 0..keys.length() {
                let key = keys
                    .get_index(scope, index)
                    .ok_or(error_internal!("unable to get item from array at index {}", index))?;

                let key: v8::Local<v8::String> = key.try_into().map_err(|_| {
                    error_internal!(
                        "get_own_property_names array item is not a string, got: {}",
                        key.type_repr()
                    )
                })?;

                let key = key.to_rust_string_lossy(scope);

                result.push(key)
            }

            result
        }
        None => vec![],
    };

    Ok(keys)
}

gauntlet_utils_macros::widget_deserialization_gen!();
