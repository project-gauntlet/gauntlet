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
#[serde(tag = "type")]
pub enum Component {
    #[serde(rename = "standard")]
    Standard {
        #[serde(rename = "internalName")]
        internal_name: String,
        name: ComponentName,
        props: Vec<Property>,
        children: Children,
    },
    #[serde(rename = "root")]
    Root {
        #[serde(rename = "internalName")]
        internal_name: String,
        children: Vec<RootChild>,
    },
    #[serde(rename = "text_part")]
    TextPart {
        #[serde(rename = "internalName")]
        internal_name: String,
    }
}

#[derive(Debug, Serialize)]
pub struct Property {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub property_type: PropertyType
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum PropertyType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "array")]
    Array {
        nested: Box<PropertyType>
    },
    #[serde(rename = "function")]
    Function,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Children {
    #[serde(rename = "members")]
    Members {
        members: Vec<ChildrenMember>,
    },
    #[serde(rename = "string")]
    String {
        component_internal_name: String,
    },
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Serialize)]
pub struct ChildrenMember {
    #[serde(rename = "memberName")]
    pub member_name: String,
    #[serde(rename = "componentInternalName")]
    pub component_internal_name: String,
    #[serde(rename = "componentName")]
    pub component_name: ComponentName,
}

#[derive(Debug, Serialize)]
pub struct RootChild {
    #[serde(rename = "componentInternalName")]
    pub component_internal_name: String,
    #[serde(rename = "componentName")]
    pub component_name: ComponentName,
}

fn children_members(members: Vec<ChildrenMember>) -> Children {
    Children::Members {
        members,
    }
}

fn children_string() -> Children {
    Children::String {
        component_internal_name: "text_part".to_owned()
    }
}

fn children_none() -> Children {
    Children::None
}

fn member(member_name: impl ToString, component: &Component) -> ChildrenMember {
    match component {
        Component::Standard { internal_name, name, .. } => {
            ChildrenMember {
                member_name: member_name.to_string(),
                component_internal_name: internal_name.to_owned(),
                component_name: name.to_owned()
            }
        }
        Component::Root { .. } => panic!("invalid component member"),
        Component::TextPart { .. } => panic!("invalid component member"),
    }
}

fn component(internal_name: impl ToString, name: impl ToString, properties: Vec<Property>, children: Children) -> Component {
    Component::Standard {
        internal_name: internal_name.to_string(),
        name: ComponentName::new(name),
        props: properties.into_iter().collect(),
        children
    }
}

fn text_part() -> Component {
    Component::TextPart {
        internal_name: "text_part".to_owned()
    }
}

fn root(children: &[&Component]) -> Component {
    Component::Root {
        internal_name: "root".to_owned(),
        children: children.into_iter()
            .map(|child| {
                match child {
                    Component::Standard { internal_name, name, .. } => {
                        RootChild { component_name: name.to_owned(), component_internal_name: internal_name.to_owned() }
                    }
                    Component::Root { .. } => panic!("invalid root child"),
                    Component::TextPart { .. } => panic!("invalid root child"),
                }
            })
            .collect(),
    }
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
        vec![],
        children_string(),
    );

    let link_component = component(
        "link",
        "Link",
        vec![
            property("href", false, PropertyType::String),
        ],
        children_string()
    );

    let tag_component = component(
        "tag",
        "Tag",
        vec![
            property("color", true, PropertyType::String),
            property("onClick", true, PropertyType::Function)
        ],
        children_string()
    );

    let metadata_item_component = component(
        "metadata_item",
        "MetadataItem",
        vec![],
        children_members(vec![
            member("Text", &text_component),
            member("Link", &link_component),
            member("Tag", &tag_component),
        ])
    );

    let separator_component = component(
        "separator",
        "Separator",
        vec![],
        children_none()
    );

    let metadata_component = component(
        "metadata",
        "Metadata",
        vec![],
        children_members(vec![
            member("Item", &metadata_item_component),
            member("Separator", &separator_component),
        ])
    );

    let image_component = component(
        "image",
        "Image",
        vec![
            // property("href", true, Type::String)
        ],
        children_none()
    );

    let h1_component = component(
        "h1",
        "H1",
        vec![],
        children_string()
    );

    let h2_component = component(
        "h2",
        "H2",
        vec![],
        children_string()
    );

    let h3_component = component(
        "h3",
        "H3",
        vec![],
        children_string()
    );

    let h4_component = component(
        "h4",
        "H4",
        vec![],
        children_string()
    );

    let h5_component = component(
        "h5",
        "H5",
        vec![],
        children_string()
    );

    let h6_component = component(
        "h6",
        "H6",
        vec![],
        children_string()
    );

    let horizontal_break_component = component(
        "horizontal_break",
        "HorizontalBreak",
        vec![],
        children_none()
    );

    let code_block_component = component(
        "code_block",
        "CodeBlock",
        vec![],
        children_string()
    );

    let code_component = component(
        "code",
        "Code",
        vec![],
        children_string()
    );

    let content_component = component(
        "content",
        "Content",
        vec![],
        children_members(vec![
            member("Text", &text_component),
            member("Link", &link_component),
            member("Image", &image_component),
            member("H1", &h1_component),
            member("H2", &h2_component),
            member("H3", &h3_component),
            member("H4", &h4_component),
            member("H5", &h5_component),
            member("H6", &h6_component),
            member("HorizontalBreak", &horizontal_break_component),
            member("CodeBlock", &code_block_component),
            member("Code", &code_component),
        ])
    );

    let detail_component = component(
        "detail",
        "Detail",
        vec![],
        children_members(vec![
            member("Metadata", &metadata_component),
            member("Content", &content_component)
        ])
    );

    let text_part = text_part();

    let root = root(&[&detail_component]);

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
        text_part,

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
        root,
    ]
}