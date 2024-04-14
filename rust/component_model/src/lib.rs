use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use indexmap::IndexMap;
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

#[derive(Debug, Clone, Serialize)]
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
        children: Vec<ComponentRef>,
        #[serde(rename = "sharedTypes")]
        shared_types: HashMap<String, SharedType>
    },
    #[serde(rename = "text_part")]
    TextPart {
        #[serde(rename = "internalName")]
        internal_name: String,
        props: Vec<Property>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Property {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub property_type: PropertyType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum PropertyType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "component")]
    Component {
        reference: ComponentRef,
    },
    #[serde(rename = "function")]
    Function {
        arguments: Vec<Property>
    },
    #[serde(rename = "image_data")]
    ImageData,
    #[serde(rename = "enum")]
    Enum {
        name: String
    },
    #[serde(rename = "union")]
    Union {
        items: Vec<PropertyType>
    },
    #[serde(rename = "object")]
    Object {
        name: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SharedType {
    #[serde(rename = "enum")]
    Enum {
        items: Vec<String>
    },
    #[serde(rename = "object")]
    Object {
        items: HashMap<String, PropertyType>
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Children {
    #[serde(rename = "string_or_members")]
    StringOrMembers {
        members: IndexMap<String, ComponentRef>,
        #[serde(rename = "textPartInternalName")]
        text_part_internal_name: String,
    },
    #[serde(rename = "members")]
    Members {
        members: IndexMap<String, ComponentRef>,
    },
    #[serde(rename = "string")]
    String {
        #[serde(rename = "textPartInternalName")]
        text_part_internal_name: String,
    },
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentRef {
    #[serde(rename = "componentInternalName")]
    pub component_internal_name: String,
    #[serde(rename = "componentName")]
    pub component_name: ComponentName,
}

fn children_string_or_members<I>(members: I) -> Children
    where I: IntoIterator<Item=(String, ComponentRef)>
{
    Children::StringOrMembers {
        text_part_internal_name: "text_part".to_owned(),
        members: members.into_iter().collect(),
    }
}

fn children_members<I>(members: I) -> Children
    where I: IntoIterator<Item=(String, ComponentRef)>
{
    Children::Members {
        members: members.into_iter().collect(),
    }
}

fn children_string() -> Children {
    Children::String {
        text_part_internal_name: "text_part".to_owned()
    }
}

fn children_none() -> Children {
    Children::None
}

fn member(member_name: impl ToString, component: &Component) -> (String, ComponentRef) {
    match component {
        Component::Standard { internal_name, name, .. } => {
            (
                member_name.to_string(),
                ComponentRef {
                    component_internal_name: internal_name.to_owned(),
                    component_name: name.to_owned(),
                }
            )
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

fn component_ref(component: &Component) -> PropertyType {
    match component {
        Component::Standard { internal_name, name, .. } => {
            PropertyType::Component {
                reference: ComponentRef {
                    component_internal_name: internal_name.to_owned(),
                    component_name: name.to_owned(),
                }
            }
        }
        Component::Root { .. } => panic!("invalid component member"),
        Component::TextPart { .. } => panic!("invalid component member"),
    }
}

fn text_part() -> Component {
    Component::TextPart {
        internal_name: "text_part".to_owned(),
        props: vec![property("value", false, PropertyType::String)]
    }
}

fn root(children: &[&Component]) -> Component {
    Component::Root {
        internal_name: "root".to_owned(),
        children: children.into_iter()
            .map(|child| {
                match child {
                    Component::Standard { internal_name, name, .. } => {
                        ComponentRef { component_name: name.to_owned(), component_internal_name: internal_name.to_owned() }
                    }
                    Component::Root { .. } => panic!("invalid root child"),
                    Component::TextPart { .. } => panic!("invalid root child"),
                }
            })
            .collect(),
        shared_types: HashMap::from([
            ("Icons".to_owned(), SharedType::Enum {
                items: [
                    "PersonAdd",
                    "Airplane",
                    // "AirplaneLanding",
                    // "AirplaneTakeoff",
                    "Alarm",
                    // "AlarmRinging",
                    "AlignCentre",
                    "AlignLeft",
                    "AlignRight",
                    // "Anchor",
                    "ArrowClockwise",
                    "ArrowCounterClockwise",
                    "ArrowDown",
                    "ArrowLeft",
                    "ArrowRight",
                    "ArrowUp",
                    "ArrowLeftRight",
                    "ArrowsContract",
                    "ArrowsExpand",
                    "AtSymbol",
                    // "BandAid",
                    "Cash",
                    // "BarChart",
                    // "BarCode",
                    "Battery",
                    "BatteryCharging",
                    // "BatteryDisabled",
                    "Bell",
                    "BellDisabled",
                    // "Bike",
                    // "Binoculars",
                    // "Bird",
                    "Document",
                    "DocumentAdd",
                    "DocumentDelete",
                    "Bluetooth",
                    // "Boat",
                    "Bold",
                    // "Bolt",
                    // "BoltDisabled",
                    "Book",
                    "Bookmark",
                    "Box",
                    // "Brush",
                    "Bug",
                    "Building",
                    "BulletPoints",
                    "Calculator",
                    "Calendar",
                    "Camera",
                    "Car",
                    "Cart",
                    // "Cd",
                    // "Center",
                    "Checkmark",
                    // "ChessPiece",
                    "ChevronDown",
                    "ChevronLeft",
                    "ChevronRight",
                    "ChevronUp",
                    "ChevronExpand",
                    "Circle",
                    // "CircleProgress100",
                    // "CircleProgress25",
                    // "CircleProgress50",
                    // "CircleProgress75",
                    // "ClearFormatting",
                    "Clipboard",
                    "Clock",
                    "Cloud",
                    "CloudLightning",
                    "CloudRain",
                    "CloudSnow",
                    "CloudSun",
                    "Code",
                    "Gear",
                    "Coin",
                    "Command",
                    "Compass",
                    // "ComputerChip",
                    // "Contrast",
                    "CreditCard",
                    "Crop",
                    // "Crown",
                    // "Desktop",
                    "Dot",
                    "Download",
                    // "Duplicate",
                    "Eject",
                    "ThreeDots",
                    "Envelope",
                    "Eraser",
                    "ExclamationMark",
                    "Eye",
                    "EyeDisabled",
                    "EyeDropper",
                    "Female",
                    "Film",
                    "Filter",
                    "Fingerprint",
                    "Flag",
                    "Folder",
                    "FolderAdd",
                    "FolderDelete",
                    "Forward",
                    "GameController",
                    "Virus",
                    "Gift",
                    "Glasses",
                    "Globe",
                    "Hammer",
                    "HardDrive",
                    "Headphones",
                    "Heart",
                    // "HeartDisabled",
                    "Heartbeat",
                    "Hourglass",
                    "House",
                    "Image",
                    "Info",
                    "Italics",
                    "Key",
                    "Keyboard",
                    "Layers",
                    // "Leaf",
                    "LightBulb",
                    "LightBulbDisabled",
                    "Link",
                    "List",
                    "Lock",
                    // "LockDisabled",
                    "LockUnlocked",
                    // "Logout",
                    // "Lowercase",
                    // "MagnifyingGlass",
                    "Male",
                    "Map",
                    "Maximize",
                    "Megaphone",
                    "MemoryModule",
                    "MemoryStick",
                    "Message",
                    "Microphone",
                    "MicrophoneDisabled",
                    "Minimize",
                    "Minus",
                    "Mobile",
                    // "Monitor",
                    "Moon",
                    // "Mountain",
                    "Mouse",
                    "Multiply",
                    "Music",
                    "Network",
                    "Paperclip",
                    "Paragraph",
                    "Pause",
                    "Pencil",
                    "Person",
                    "Phone",
                    // "PhoneRinging",
                    "PieChart",
                    "Capsule",
                    // "Pin",
                    // "PinDisabled",
                    "Play",
                    "Plug",
                    "Plus",
                    // "PlusMinusDivideMultiply",
                    "Power",
                    "Printer",
                    "QuestionMark",
                    "Quotes",
                    "Receipt",
                    "PersonRemove",
                    "Repeat",
                    "Reply",
                    "Rewind",
                    "Rocket",
                    // "Ruler",
                    "Shield",
                    "Shuffle",
                    "Snippets",
                    "Snowflake",
                    // "VolumeHigh",
                    // "VolumeLow",
                    // "VolumeOff",
                    // "VolumeOn",
                    "Star",
                    // "StarDisabled",
                    "Stop",
                    "Stopwatch",
                    "StrikeThrough",
                    "Sun",
                    // "Syringe",
                    "Scissors",
                    "Tag",
                    "Thermometer",
                    "Terminal",
                    "Text",
                    "TextCursor",
                    // "TextSelection",
                    // "Torch",
                    // "Train",
                    "Trash",
                    "Tree",
                    "Trophy",
                    "People",
                    "Umbrella",
                    "Underline",
                    "Upload",
                    // "Uppercase",
                    "Wallet",
                    "Wand",
                    // "Warning",
                    // "Weights",
                    "Wifi",
                    "WifiDisabled",
                    "Window",
                    "Tools",
                    "Watch",
                    "XMark",
                    //
                    "Indent",
                    "Unindent",

                ].into_iter().map(|s| s.to_string()).collect()
            }),
            (
                "ImageSource".to_owned(),
                SharedType::Object { items: HashMap::from([("data".to_owned(), PropertyType::ImageData)]) }
            )
        ]),
    }
}

fn event<I>(name: impl Into<String>, optional: bool, arguments: I) -> Property
    where I: IntoIterator<Item=Property>
{
    Property {
        name: name.into(),
        optional,
        property_type: PropertyType::Function { arguments: arguments.into_iter().collect() },
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

    let action_component = component(
        "action",
        "Action",
        [
            property("id", true, PropertyType::String),
            property("title", false, PropertyType::String),
            event("onAction", false, [])
        ],
        children_none(),
    );


    let action_panel_section_component = component(
        "action_panel_section",
        "ActionPanelSection",
        [
            property("title", true, PropertyType::String),
        ],
        children_members([
            member("Action", &action_component),
        ]),
    );


    let action_panel_component = component(
        "action_panel",
        "ActionPanel",
        [
            property("title", true, PropertyType::String),
        ],
        children_members([
            member("Action", &action_component),
            member("Section", &action_panel_section_component),
        ]),
    );


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
            event("onClick", true, [])
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
            property("icon", false, PropertyType::Enum { name: "Icons".to_owned() }),
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

    // let link_component = component(
    //     "link",
    //     "Link",
    //     [
    //         property("href", false, PropertyType::String),
    //     ],
    //     children_string(),
    // );

    let image_component = component(
        "image",
        "Image",
        [
            property("source", false, PropertyType::Object { name: "ImageSource".to_owned() })
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

    // content shouldn't have any interactable items
    let content_component = component(
        "content",
        "Content",
        [],
        children_members([
            member("Paragraph", &paragraph_component),
            // member("Link", &link_component),
            member("Image", &image_component), // TODO color
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
        [
            property("actions", true, component_ref(&action_panel_component))
        ],
        children_members([
            member("Metadata", &metadata_component),
            member("Content", &content_component),
        ]),
    );


    let text_field_component = component(
        "text_field",
        "TextField",
        [
            property("label", true, PropertyType::String),
            property("value", true, PropertyType::String),
            event("onChange", true, [property("value", true, PropertyType::String)])
        ],
        children_none(),
    );

    let password_field_component = component(
        "password_field",
        "PasswordField",
        [
            property("label", true, PropertyType::String),
            property("value", true, PropertyType::String),
            event("onChange", true, [property("value", true, PropertyType::String)])
        ],
        children_none(),
    );

    // let text_area_component = component(
    //     "text_area",
    //     "TextArea",
    //     [],
    //     children_none(),
    // );

    let checkbox_component = component(
        "checkbox",
        "Checkbox",
        [
            property("label", true, PropertyType::String),
            property("title", true, PropertyType::String),
            property("value", true, PropertyType::Boolean),
            event("onChange", true, [property("value", false, PropertyType::Boolean)])
        ],
        children_none(),
    );

    let date_picker_component = component(
        "date_picker",
        "DatePicker",
        [
            property("label", true, PropertyType::String),
            property("value", true, PropertyType::String),
            event("onChange", true, [property("value", true, PropertyType::String)])
        ],
        children_none(),
    );

    let select_item_component = component(
        "select_item",
        "SelectItem",
        [
            property("value", false, PropertyType::String),
        ],
        children_string(),
    );

    let select_component = component(
        "select",
        "Select",
        [
            property("label", true, PropertyType::String),
            property("value", true, PropertyType::String),
            event("onChange", true, [property("value", true, PropertyType::String)])
        ],
        children_members([
            member("Item", &select_item_component)
        ]),
    );

    // let multi_select_component = component(
    //     "multi_select",
    //     "MultiSelect",
    //     [],
    //     children_none(),
    // );

    let separator_component = component(
        "separator",
        "Separator",
        [],
        children_none(),
    );

    let form_component = component(
        "form",
        "Form",
        [
            property("actions", true, component_ref(&action_panel_component)),
        ],
        children_members([
            member("TextField", &text_field_component),
            member("PasswordField", &password_field_component),
            // member("TextArea", &text_area_component),
            member("Checkbox", &checkbox_component),
            member("DatePicker", &date_picker_component),
            member("Select", &select_component),
            // member("MultiSelect", &multi_select_component),
            member("Separator", &separator_component),
        ]),
    );

    let inline_separator_component = component(
        "inline_separator",
        "InlineSeparator",
        [
            property("icon", true, PropertyType::Enum { name: "Icons".to_owned() }),
        ],
        children_none(),
    );

    let inline_component = component(
        "inline",
        "Inline",
        [],
        children_members([
            member("Left", &content_component),
            member("Separator", &inline_separator_component),
            member("Right", &content_component),
            member("Center", &content_component),
        ]),
    );

    let empty_view_component = component(
        "empty_view",
        "EmptyView",
        [
            property("title", false, PropertyType::String),
            property("description", true, PropertyType::String),
            property("image", true, PropertyType::Object { name: "ImageSource".to_owned() }),
        ],
        children_none(),
    );

    let list_item_component = component(
        "list_item",
        "ListItem",
        [
            property("id", false, PropertyType::String),
            property("title", false, PropertyType::String),
            property("subtitle", true, PropertyType::String),
            property("icon", true, PropertyType::Union { items: vec![PropertyType::Object { name: "ImageSource".to_owned() }, PropertyType::Enum { name: "Icons".to_owned() }] }),
            // accessories
        ],
        children_none(),
    );

    let list_section_component = component(
        "list_section",
        "ListSection",
        [
            property("title", false, PropertyType::String),
            property("subtitle", true, PropertyType::String),
        ],
        children_members([
            member("Item", &list_item_component),
        ]),
    );

    let list_component = component(
        "list",
        "List",
        [
            property("actions", true, component_ref(&action_panel_component)),
            event("onSelectionChange", true, [property("id", false, PropertyType::String)]),
        ],
        children_members([
            member("EmptyView", &empty_view_component),
            member("Detail", &detail_component),
            member("Item", &list_item_component),
            member("Section", &list_section_component),
        ]),
    );

    let grid_item_component = component(
        "grid_item",
        "GridItem",
        [
            property("id", false, PropertyType::String),
            property("title", false, PropertyType::String),
            property("subtitle", true, PropertyType::String),
            // accessories
        ],
        children_members([
            member("Content", &content_component),
        ]),
    );

    let grid_section_component = component(
        "grid_section",
        "GridSection",
        [
            property("title", false, PropertyType::String),
            property("subtitle", true, PropertyType::String),
            // property("aspectRatio", true, PropertyType::String),
            property("columns", true, PropertyType::Number)
            // fit
            // inset
        ],
        children_members([
            member("Item", &grid_item_component),
        ]),
    );

    let grid_component = component(
        "grid",
        "Grid",
        [
            property("actions", true, component_ref(&action_panel_component)),
            // property("aspectRatio", true, PropertyType::String),
            property("columns", true, PropertyType::Number), // TODO default
            // fit
            // inset
            event("onSelectionChange", true, [property("id", false, PropertyType::String)]),
        ],
        children_members([
            member("EmptyView", &empty_view_component),
            member("Detail", &detail_component),
            member("Item", &grid_item_component),
            member("Section", &grid_section_component),
        ]),
    );

    let text_part = text_part();

    let root = root(&[
        &detail_component,
        &form_component,
        &inline_component,
        &list_component,
        &grid_component,
    ]);

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

    // Inline
    // Inline.Left
    // Inline.Separator
    // Inline.Right
    // Inline.Center

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

        action_component,
        action_panel_section_component,
        action_panel_component,

        metadata_link_component,
        metadata_tag_item_component,
        metadata_tag_list_component,
        metadata_separator_component,
        metadata_value_component,
        metadata_icon_component,
        metadata_component,

        // link_component,
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

        text_field_component,
        password_field_component,
        // text_area_component,
        checkbox_component,
        date_picker_component,
        select_item_component,
        select_component,
        // multi_select_component,
        separator_component,
        form_component,

        inline_separator_component,
        inline_component,

        empty_view_component,
        list_item_component,
        list_section_component,
        list_component,
        grid_item_component,
        grid_section_component,
        grid_component,

        root,
    ]
}