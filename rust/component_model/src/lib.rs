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
    },
}

#[derive(Debug, Serialize)]
pub struct Property {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub property_type: PropertyType,
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
    #[serde(rename = "string_or_members")]
    StringOrMembers {
        members: Vec<ChildrenMember>,
        component_internal_name: String,
    },
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

fn children_string_or_members<I>(members: I) -> Children
    where I: IntoIterator<Item=ChildrenMember>
{
    Children::StringOrMembers {
        component_internal_name: "text_part".to_owned(),
        members: members.into_iter().collect(),
    }
}

fn children_members<I>(members: I) -> Children
    where I: IntoIterator<Item=ChildrenMember>
{
    Children::Members {
        members: members.into_iter().collect(),
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
                component_name: name.to_owned(),
            }
        }
        Component::Root { .. } => panic!("invalid component member"),
        Component::TextPart { .. } => panic!("invalid component member"),
    }
}


fn component<I>(internal_name: impl ToString, name: impl ToString, properties: I, children: Children) -> Component
    where I: IntoIterator<Item=Property>
{
    Component::Standard {
        internal_name: internal_name.to_string(),
        name: ComponentName::new(name),
        props: properties.into_iter().collect(),
        children,
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
    let metadata_link_component = component(
        "metadata_link",
        "MetadataLink",
        [
            property("label", false, PropertyType::String),
            property("href", false, PropertyType::String),
        ],
        children_string(),
    );

    let metadata_tag_item_component = component(
        "metadata_tag_item",
        "MetadataTagItem",
        [
            // property("color", true, PropertyType::String),
            property("onClick", true, PropertyType::Function)
        ],
        children_string(),
    );

    let metadata_tag_list_component = component(
        "metadata_tag_list",
        "MetadataTagList",
        [
            property("label", false, PropertyType::String)
        ],
        children_members([
            member("Item", &metadata_tag_item_component),
        ]),
    );

    let metadata_separator_component = component(
        "metadata_separator",
        "MetadataSeparator",
        [],
        children_none(),
    );

    let metadata_icon_component = component(
        "metadata_icon",
        "MetadataIcon",
        [
            property("icon", false, PropertyType::String),
            property("label", false, PropertyType::String),
        ],
        children_none(),
    );

    let metadata_value_component = component(
        "metadata_value",
        "MetadataValue",
        [
            property("label", false, PropertyType::String),
        ],
        children_string(),
    );

    let metadata_component = component(
        "metadata",
        "Metadata",
        [],
        children_members([
            member("TagList", &metadata_tag_list_component),
            member("Link", &metadata_link_component),
            member("Value", &metadata_value_component),
            member("Icon", &metadata_icon_component),
            member("Separator", &metadata_separator_component),
        ]),
    );

    let link_component = component(
        "link",
        "Link",
        [
            property("href", false, PropertyType::String),
        ],
        children_string(),
    );

    let image_component = component(
        "image",
        "Image",
        [
            // property("href", true, Type::String)
        ],
        children_none(),
    );

    let h1_component = component(
        "h1",
        "H1",
        [],
        children_string(),
    );

    let h2_component = component(
        "h2",
        "H2",
        [],
        children_string(),
    );

    let h3_component = component(
        "h3",
        "H3",
        [],
        children_string(),
    );

    let h4_component = component(
        "h4",
        "H4",
        [],
        children_string(),
    );

    let h5_component = component(
        "h5",
        "H5",
        [],
        children_string(),
    );

    let h6_component = component(
        "h6",
        "H6",
        [],
        children_string(),
    );

    let horizontal_break_component = component(
        "horizontal_break",
        "HorizontalBreak",
        [],
        children_none(),
    );

    let code_block_component = component(
        "code_block",
        "CodeBlock",
        [],
        children_string(),
    );

    // let code_component = component(
    //     "code",
    //     "Code",
    //     [],
    //     children_string()
    // );

    let paragraph_component = component(
        "paragraph",
        "Paragraph",
        [],
        children_string(),
        // children_string_or_members([
        //     member("Link", &link_component),
        //     member("Code", &code_component),
        // ]),
    );

    let content_component = component(
        "content",
        "Content",
        [],
        children_members([
            member("Paragraph", &paragraph_component),
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
            // member("Code", &code_component),
        ]),
    );

    let detail_component = component(
        "detail",
        "Detail",
        [],
        children_members([
            member("Metadata", &metadata_component),
            member("Content", &content_component),
        ]),
    );

    let text_part = text_part();

    let root = root(&[&detail_component]);

    // Detail
    // Detail.Content
    // Detail.Content.Link
    // Detail.Content.Code
    // Detail.Content.Paragraph
    // Detail.Content.Paragraph.Link
    // Detail.Content.Paragraph.Code
    // Detail.Content.Image
    // Detail.Content.H1-6
    // Detail.Content.HorizontalBreak
    // Detail.Content.CodeBlock
    // Detail.Metadata
    // Detail.Metadata.TagList
    // Detail.Metadata.TagList.Item
    // Detail.Metadata.Separator
    // Detail.Metadata.Link
    // Detail.Metadata.Value
    // Detail.Metadata.Icon

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
        metadata_link_component,
        metadata_tag_item_component,
        metadata_tag_list_component,
        metadata_separator_component,
        metadata_value_component,
        metadata_icon_component,
        metadata_component,
        link_component,
        image_component,
        h1_component,
        h2_component,
        h3_component,
        h4_component,
        h5_component,
        h6_component,
        horizontal_break_component,
        code_block_component,
        // code_component,
        paragraph_component,
        content_component,
        detail_component,
        root,
    ]
}