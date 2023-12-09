use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentName(Arc<str>);

impl ComponentName {
    pub fn new(value: impl ToString) -> Self {
        ComponentName(value.to_string().into())
    }
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
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

    pub fn name(&self) -> &str {
        &self.name.0
    }

    pub fn props(&self) -> &[Property] {
        &self.props
    }

    pub fn members(&self) -> &HashMap<String, ComponentName> {
        &self.members
    }
}

#[derive(Debug, Serialize)]
pub struct Property {
    name: String,
    optional: bool,
    #[serde(rename = "type")]
    property_type: PropertyType
}

impl Property {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn optional(&self) -> bool {
        self.optional
    }

    pub fn property_type(&self) -> &PropertyType {
        &self.property_type
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "name")]
pub enum PropertyType {
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
        nested: Box<PropertyType>
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
            .collect(),
    }
}

fn container(children_components: &[&ComponentName]) -> Component {
    Component {
        internal_name: "container".to_string(),
        name: ComponentName::new("Container"),
        props: vec![
            children(PropertyType::Components {
                components: children_components.iter().map(|x| ComponentName::new(x.to_string())).collect()
            })
        ],
        members: HashMap::new(),
    }
}

fn children(property_type: PropertyType) -> Property {
    match property_type {
        PropertyType::Components { .. } => {}
        PropertyType::StringComponent => {}
        _ => {
            panic!("{:?} property_type not supported as children", property_type)
        }
    }
    property("children", true, property_type)
}


fn property(name: impl Into<String>, optional: bool, property_type: PropertyType) -> Property {
    Property {
        name: name.into(),
        optional,
        property_type,
    }
}

pub fn create_component_model() -> Vec<Component> {
    let text_component = component(
        "text",
        "Text",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[],
    );

    let link_component = component(
        "link",
        "Link",
        vec![
            property("href", false, PropertyType::String),
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let tag_component = component(
        "tag",
        "Tag",
        vec![
            children(PropertyType::StringComponent),
            property("color", true, PropertyType::String),
            property("onClick", true, PropertyType::Function)
        ],
        &[]
    );

    let metadata_item_component = component(
        "metadata_item",
        "MetadataItem",
        vec![
            children(PropertyType::Components {
                components: vec![
                    text_component.name.clone(),
                    link_component.name.clone(),
                    tag_component.name.clone(),
                ]
            })
        ],
        &[
            ("Text", &text_component.name),
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
            children(PropertyType::Components {
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
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let h2_component = component(
        "h2",
        "H2",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let h3_component = component(
        "h3",
        "H3",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let h4_component = component(
        "h4",
        "H4",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let h5_component = component(
        "h5",
        "H5",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let h6_component = component(
        "h6",
        "H6",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let horizontal_break_component = component(
        "horizontal_break",
        "HorizontalBreak",
        vec![],
        &[]
    );

    let code_block_component = component(
        "code_block",
        "CodeBlock",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let code_component = component(
        "code",
        "Code",
        vec![
            children(PropertyType::StringComponent)
        ],
        &[]
    );

    let content_component = component(
        "content",
        "Content",
        vec![
            children(PropertyType::Components {
                components: vec![
                    text_component.name.clone(),
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
            ("Text", &text_component.name),
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
            children(PropertyType::Components {
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

    let container = container(&[
        &detail_component.name,
    ]);

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
        text_component,
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

        container,
    ]
}