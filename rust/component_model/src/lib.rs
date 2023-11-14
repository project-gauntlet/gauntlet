use std::collections::HashMap;

use serde::Serialize;

type ComponentName = String;

#[derive(Debug, Serialize)]
pub struct Component {
    #[serde(rename = "internalName")]
    internal_name: String,
    name: ComponentName,
    props: Vec<Property>,
    members: HashMap<String, ComponentName>,
}

#[derive(Debug, Serialize)]
pub struct Property {
    name: String,
    optional: bool,
    #[serde(rename = "type")]
    property_type: Type
}

#[derive(Debug, Serialize)]
#[serde(tag = "name")]
pub enum Type {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "reactnode")]
    ReactNode,
    #[serde(rename = "array")]
    Array {
        nested: Box<Type>
    },
    #[serde(rename = "or")]
    Or {
        nested: Vec<Type>
    },
    #[serde(rename = "function")]
    Function,
}

fn component(internal_name: impl Into<String>, name: impl Into<String>, properties: Vec<Property>, children: &[(&str, &str)]) -> Component {
    Component {
        internal_name: internal_name.into(),
        name: name.into(),
        props: properties,
        members: children.into_iter()
            .map(|&(key, value)| (key.to_owned(), value.to_owned()))
            .collect()
    }
}

fn property(name: impl Into<String>, optional: bool, property_type: Type) -> Property {
    Property {
        name: name.into(),
        optional,
        property_type,
    }
}

pub fn create_component_model() -> Vec<Component> {
    let text_inner_component = component(
        "text_inner",
        "TextInner",
        vec![
            property("children", false, Type::Or { nested: vec![Type::ReactNode]})
        ],
        &[]
    );

    let button_component = component(
        "button",
        "Button",
        vec![
            property("onClick", true, Type::Function),
            property("children", true, Type::String)
        ],
        &[]
    );

    let box_component = component(
        "box",
        "Box",
        vec![
            property("test", false, Type::Number),
            property("children", true, Type::ReactNode)
        ],
        &[
            ("Text", &text_inner_component.name),
            ("Button", &button_component.name)
        ]
    );

    vec![text_inner_component, button_component, box_component]
}