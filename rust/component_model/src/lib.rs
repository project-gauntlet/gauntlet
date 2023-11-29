use std::collections::HashMap;
use std::sync::Arc;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentName(Arc<str>);

impl ComponentName {
    pub fn new(value: impl ToString) -> Self {
        ComponentName(value.to_string().into())
    }
}

impl ToString for ComponentName {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Serialize for ComponentName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}


#[derive(Debug, Serialize)]
pub struct Component {
    #[serde(rename = "internalName")]
    internal_name: String,
    name: ComponentName,
    props: Vec<Property>,
    members: HashMap<String, ComponentName>,
}

impl Component {
    pub fn internal_name(&self) -> &str {
        &self.internal_name
    }
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
    #[serde(rename = "components")]
    Components {
        components: Vec<ComponentName>,
    },
    #[serde(rename = "stringcomponent")]
    StringComponent,
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

fn component(internal_name: impl ToString, name: impl ToString, properties: Vec<Property>, children: &[(&str, &ComponentName)]) -> Component {
    Component {
        internal_name: internal_name.to_string(),
        name: ComponentName::new(name),
        props: properties,
        members: children.into_iter()
            .map(|&(key, value)| (key.to_owned(), value.clone()))
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
    let text_content_component = component(
        "textcontent",
        "Text",
        vec![
            property("children", true, Type::StringComponent)
        ],
        &[],
    );

    let link_component = component(
        "link",
        "Link",
        vec![
            property("href", false, Type::String),
            property("children", false, Type::String)
        ],
        &[]
    );

    let tag_component = component(
        "tag",
        "Tag",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let metadata_item_component = component(
        "metadata_item",
        "MetadataItem",
        vec![
            property("children", false, Type::Components {
                components: vec![
                    text_content_component.name.clone(),
                    link_component.name.clone(),
                    tag_component.name.clone(),
                ]
            })
        ],
        &[
            ("Text", &text_content_component.name),
            ("Link", &link_component.name),
            ("Tag", &tag_component.name),
        ]
    );

    let separator_component = component(
        "separator",
        "Separator",
        vec![],
        &[]
    );

    let metadata_component = component(
        "metadata",
        "Metadata",
        vec![
            property("children", false, Type::Components {
                components: vec![
                    metadata_item_component.name.clone(),
                    separator_component.name.clone(),
                ]
            })
        ],
        &[
            ("Item", &metadata_item_component.name),
            ("Separator", &separator_component.name),
        ]
    );

    let image_component = component(
        "image",
        "Image",
        vec![
            // property("href", true, Type::String)
        ],
        &[]
    );

    let h1_component = component(
        "h1",
        "H1",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let h2_component = component(
        "h2",
        "H2",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let h3_component = component(
        "h3",
        "H3",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let h4_component = component(
        "h4",
        "H4",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let h5_component = component(
        "h5",
        "H5",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let h6_component = component(
        "h6",
        "H6",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let horizontal_break_component = component(
        "horizontal_break",
        "HorizontalBreak",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let code_block_component = component(
        "code_block",
        "CodeBlock",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let code_component = component(
        "code",
        "Code",
        vec![
            property("children", true, Type::String)
        ],
        &[]
    );

    let content_component = component(
        "content",
        "Content",
        vec![
            property("children", true, Type::Components {
                components: vec![
                    text_content_component.name.clone(),
                    link_component.name.clone(),
                    image_component.name.clone(),
                    h1_component.name.clone(),
                    h2_component.name.clone(),
                    h3_component.name.clone(),
                    h4_component.name.clone(),
                    h5_component.name.clone(),
                    h6_component.name.clone(),
                    horizontal_break_component.name.clone(),
                    code_block_component.name.clone(),
                    code_component.name.clone(),
                ]
            })
        ],
        &[
            ("Text", &text_content_component.name),
            ("Link", &link_component.name),
            ("Image", &image_component.name),
            ("H1", &h1_component.name),
            ("H2", &h2_component.name),
            ("H3", &h3_component.name),
            ("H4", &h4_component.name),
            ("H5", &h5_component.name),
            ("H6", &h6_component.name),
            ("HorizontalBreak", &horizontal_break_component.name),
            ("CodeBlock", &code_block_component.name),
            ("Code", &code_component.name),
        ]
    );

    let detail_component = component(
        "detail",
        "Detail",
        vec![
            property("children", true, Type::Components {
                components: vec![
                    metadata_component.name.clone(),
                    content_component.name.clone(),
                ]
            })
        ],
        &[
            ("Metadata", &metadata_component.name),
            ("Content", &content_component.name)
        ]
    );

    // Detail
    // Detail.Content
    // Detail.Content.Text
    // Detail.Content.Link
    // Detail.Content.Image
    // Detail.Content.H1-6
    // Detail.Content.HorizontalBreak
    // Detail.Content.CodeBlock
    // Detail.Content.Code
    // Detail.Metadata.Item -- label, icon
    // Detail.Metadata.Item.Text
    // Detail.Metadata.Item.Link
    // Detail.Metadata.Item.Tag
    // Detail.Metadata.Separator

    // ActionPanel
    // ActionPanel.Section
    // ActionPanel.SubMenu

    // Action

    // Form
    // Form.TextField
    // Form.PasswordField
    // Form.TextArea
    // Form.Checkbox
    // Form.DatePicker
    // Form.Dropdown
    // Form.Dropdown.Item
    // Form.Dropdown.Section
    // Form.TagPicker
    // Form.TagPicker.Item
    // Form.Separator
    // Form.FilePicker
    // Form.Description

    // List
    // List.Dropdown
    // List.Dropdown.Item
    // List.Dropdown.Section
    // List.EmptyView
    // List.Item
    // List.Item.Detail = Detail
    // List.Section

    // Grid
    // Grid.Dropdown
    // Grid.Dropdown.Item
    // Grid.Dropdown.Section
    // Grid.EmptyView
    // Grid.Item
    // Grid.Section

    vec![
        text_content_component,
        link_component,

        tag_component,
        metadata_item_component,
        separator_component,
        metadata_component,

        image_component,
        h1_component,
        h2_component,
        h3_component,
        h4_component,
        h5_component,
        h6_component,
        horizontal_break_component,
        code_block_component,
        code_component,
        content_component,

        detail_component,
    ]
}