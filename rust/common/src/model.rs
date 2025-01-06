use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use bincode::{Decode, Encode};
use gix_url::Scheme;
use gix_url::Url;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct PluginId(Arc<str>);

impl PluginId {
    pub fn from_string(plugin_id: impl ToString) -> Self {
        PluginId(plugin_id.to_string().into())
    }

    fn try_to_url(&self) -> anyhow::Result<Url> {
        let url = self.to_string();
        let url: &str = url.as_ref();
        let url = gix_url::parse(url.try_into()?)?;
        Ok(url)
    }

    pub fn try_to_git_url(&self) -> anyhow::Result<String> {
        let url = self.try_to_url()?;

        match url.scheme {
            Scheme::Git | Scheme::Ssh | Scheme::Http | Scheme::Https => {
                Ok(url.to_bstring().to_string())
            }
            Scheme::File => {
                Err(anyhow!("'file' schema is not supported"))
            }
            Scheme::Ext(schema) => {
                Err(anyhow!("'{}' schema is not supported", schema))
            }
        }
    }

    pub fn try_to_path(&self) -> anyhow::Result<PathBuf> {
        let url = self.try_to_url()?;

        if url.scheme != Scheme::File {
            return Err(anyhow!("plugin id is expected to point to local file"))
        }

        let plugin_dir: String = url.path.try_into()?;
        let plugin_dir = PathBuf::from(plugin_dir);
        Ok(plugin_dir)
    }
}

impl ToString for PluginId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct EntrypointId(Arc<str>);

impl EntrypointId {
    pub fn from_string(entrypoint_id: impl ToString) -> Self {
        EntrypointId(entrypoint_id.to_string().into())
    }
}

impl ToString for EntrypointId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    InProgress,
    Done,
    Failed {
        message: String
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UiRenderLocation {
    InlineView,
    View
}

#[derive(Debug, Clone)]
pub struct PhysicalShortcut {
    pub physical_key: PhysicalKey,
    pub modifier_shift: bool,
    pub modifier_control: bool,
    pub modifier_alt: bool,
    pub modifier_meta: bool,
}

#[derive(Debug, Clone)]
pub struct LocalSaveData {
    pub stdout_file_path: String,
    pub stderr_file_path: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub entrypoint_generator_name: Option<String>,
    pub entrypoint_icon: Option<String>,
    pub entrypoint_type: SearchResultEntrypointType,
    pub entrypoint_actions: Vec<SearchResultEntrypointAction>,
    pub entrypoint_accessories: Vec<SearchResultAccessory>,
}

#[derive(Debug, Clone)]
pub enum SearchResultAccessory {
    TextAccessory {
        text: String,
        icon: Option<Icons>,
        tooltip: Option<String>
    },
    IconAccessory {
        icon: Icons,
        tooltip: Option<String>
    },
}

#[derive(Debug, Clone)]
pub struct SearchResultEntrypointAction {
    pub action_type: SearchResultEntrypointActionType,
    pub label: String,
    pub shortcut: Option<PhysicalShortcut>,
}

#[derive(Debug, Clone)]
pub enum SearchResultEntrypointActionType {
    Command,
    View,
}

#[derive(Debug, Clone)]
pub enum SearchResultEntrypointType {
    Command,
    View,
    Generated,
}

#[derive(Debug, Clone)]
pub enum UiThemeMode {
    Light,
    Dark
}

#[derive(Debug, Clone)]
pub struct UiThemeColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub type UiThemeColorPalette = [UiThemeColor; 4];

#[derive(Debug, Clone)]
pub struct UiThemeWindow {
    pub border: UiThemeWindowBorder,
}

#[derive(Debug, Clone)]
pub struct UiThemeWindowBorder {
    pub radius: f32,
    pub width: f32,
    pub color: UiThemeColor,
}

#[derive(Debug, Clone)]
pub struct UiThemeContent {
    pub border: UiThemeContentBorder,
}

#[derive(Debug, Clone)]
pub struct UiThemeContentBorder {
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct UiTheme {
    pub mode: UiThemeMode,
    pub background: UiThemeColorPalette,
    pub text: UiThemeColorPalette,
    pub window: UiThemeWindow,
    pub content: UiThemeContent,
}

#[derive(Debug)]
pub struct UiSetupData {
    pub theme: UiTheme,
    pub global_shortcut: Option<PhysicalShortcut>,
}

#[derive(Debug)]
pub struct UiSetupResponse {
    pub global_shortcut_error: Option<String>,
}

#[derive(Debug)]
pub enum UiResponseData {
    Nothing,
    Err(anyhow::Error),
}

#[derive(Debug)]
pub enum UiRequestData {
    ShowWindow,
    ClearInlineView {
        plugin_id: PluginId
    },
    ReplaceView {
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
        images: HashMap<UiWidgetId, Vec<u8>>,
    },
    ShowPreferenceRequiredView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    ShowPluginErrorView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    },
    RequestSearchResultUpdate,
    ShowHud {
        display: String
    },
    UpdateLoadingBar {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        show: bool
    },
    SetGlobalShortcut {
        shortcut: Option<PhysicalShortcut>
    },
    SetTheme {
        theme: UiTheme
    },
}

#[derive(Debug)]
pub enum BackendResponseData {
    Nothing,
    SetupData {
        data: UiSetupData
    },
    Search {
        results: Vec<SearchResult>
    },
    RequestViewRender {
        shortcuts: HashMap<String, PhysicalShortcut>
    },
    InlineViewShortcuts {
        shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>
    },
}

#[derive(Debug)]
pub enum BackendRequestData {
    Setup,
    Search {
        text: String,
        render_inline_view: bool
    },
    RequestViewRender {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    },
    RequestViewClose {
        plugin_id: PluginId,
    },
    RequestRunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    },
    RequestRunGeneratedCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize
    },
    SendViewEvent {
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>
    },
    SendKeyboardEvent {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        origin: KeyboardEventOrigin,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    },
    SendOpenEvent {
        plugin_id: PluginId,
        href: String
    },
    OpenSettingsWindow,
    OpenSettingsWindowPreferences {
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>
    },
    InlineViewShortcuts,
    SetupResponse {
        global_shortcut_error: Option<String>
    },
}

#[derive(Debug, Clone)]
pub enum KeyboardEventOrigin {
    MainView,
    PluginView,
}

fn option_to_array<S, V>(value: &Option<V>, serializer: S) -> Result<S::Ok, S::Error>
where
    V: Serialize,
    S: Serializer,
{
    let value = match value {
        None => vec![],
        Some(value) => vec![value]
    };

    let res = Vec::<&V>::serialize(&value, serializer)?;

    Ok(res)
}

fn array_to_option<'de, D, V>(deserializer: D) -> Result<Option<V>, D::Error> where D: Deserializer<'de>, V: Deserialize<'de> {
    let res = Option::<Vec<V>>::deserialize(deserializer)?;

    match res {
        None => Ok(None),
        Some(mut res) => {
            match res.len() {
                0 => Ok(None),
                1 => Ok(Some(res.remove(0))),
                _ => Err(Error::custom("only zero or one allowed"))
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/components.rs"));


// TODO generate this
#[allow(async_fn_in_trait)]
pub trait WidgetVisitor {
    async fn action_widget(&mut self, _widget: &ActionWidget) {
    }
    async fn action_panel_section_widget(&mut self, widget: &ActionPanelSectionWidget) {
        for members in &widget.content.ordered_members {
            match members {
                ActionPanelSectionWidgetOrderedMembers::Action(widget) => self.action_widget(widget).await
            }
        }
    }
    async fn action_panel_widget(&mut self, widget: &ActionPanelWidget) {
        for members in &widget.content.ordered_members {
            match members {
                ActionPanelWidgetOrderedMembers::Action(widget) => self.action_widget(widget).await,
                ActionPanelWidgetOrderedMembers::ActionPanelSection(widget) => self.action_panel_section_widget(widget).await
            }
        }
    }

    async fn metadata_link_widget(&mut self, _widget: &MetadataLinkWidget) {}
    async fn metadata_tag_item_widget(&mut self, _widget: &MetadataTagItemWidget) {}
    async fn metadata_tag_list_widget(&mut self, widget: &MetadataTagListWidget) {
        for members in &widget.content.ordered_members {
            match members {
                MetadataTagListWidgetOrderedMembers::MetadataTagItem(widget) => self.metadata_tag_item_widget(widget).await
            }
        }
    }
    async fn metadata_separator_widget(&mut self, _widget: &MetadataSeparatorWidget) {}
    async fn metadata_value_widget(&mut self, _widget: &MetadataValueWidget) {}
    async fn metadata_icon_widget(&mut self, _widget: &MetadataIconWidget) {}
    async fn metadata_widget(&mut self, widget: &MetadataWidget) {
        for members in &widget.content.ordered_members {
            match members {
                MetadataWidgetOrderedMembers::MetadataTagList(widget) => self.metadata_tag_list_widget(widget).await,
                MetadataWidgetOrderedMembers::MetadataLink(widget) => self.metadata_link_widget(widget).await,
                MetadataWidgetOrderedMembers::MetadataValue(widget) => self.metadata_value_widget(widget).await,
                MetadataWidgetOrderedMembers::MetadataIcon(widget) => self.metadata_icon_widget(widget).await,
                MetadataWidgetOrderedMembers::MetadataSeparator(widget) => self.metadata_separator_widget(widget).await,
            }
        }
    }

    async fn image(&mut self, _widget_id: UiWidgetId, _widget: &ImageLike) {

    }

    async fn image_widget(&mut self, widget: &ImageWidget) {
        self.image(widget.__id__, &widget.source).await
    }
    async fn h1_widget(&mut self, _widget: &H1Widget) {}
    async fn h2_widget(&mut self, _widget: &H2Widget) {}
    async fn h3_widget(&mut self, _widget: &H3Widget) {}
    async fn h4_widget(&mut self, _widget: &H4Widget) {}
    async fn h5_widget(&mut self, _widget: &H5Widget) {}
    async fn h6_widget(&mut self, _widget: &H6Widget) {}
    async fn horizontal_break_widget(&mut self, _widget: &HorizontalBreakWidget) {}
    async fn code_block_widget(&mut self, _widget: &CodeBlockWidget) {}
    async fn paragraph_widget(&mut self, _widget: &ParagraphWidget) {}
    async fn content_widget(&mut self, widget: &ContentWidget) {
        for members in &widget.content.ordered_members {
            match members {
                ContentWidgetOrderedMembers::Paragraph(widget) => self.paragraph_widget(widget).await,
                ContentWidgetOrderedMembers::Image(widget) => self.image_widget(widget).await,
                ContentWidgetOrderedMembers::H1(widget) => self.h1_widget(widget).await,
                ContentWidgetOrderedMembers::H2(widget) => self.h2_widget(widget).await,
                ContentWidgetOrderedMembers::H3(widget) => self.h3_widget(widget).await,
                ContentWidgetOrderedMembers::H4(widget) => self.h4_widget(widget).await,
                ContentWidgetOrderedMembers::H5(widget) => self.h5_widget(widget).await,
                ContentWidgetOrderedMembers::H6(widget) => self.h6_widget(widget).await,
                ContentWidgetOrderedMembers::HorizontalBreak(widget) => self.horizontal_break_widget(widget).await,
                ContentWidgetOrderedMembers::CodeBlock(widget) => self.code_block_widget(widget).await,
            }
        }
    }

    async fn detail_widget(&mut self, widget: &DetailWidget) {
        if let Some(widget) = &widget.content.actions {
            self.action_panel_widget(widget).await
        }
        if let Some(widget) = &widget.content.metadata {
            self.metadata_widget(widget).await
        }
        if let Some(widget) = &widget.content.content {
            self.content_widget(widget).await
        }
    }

    async fn text_field_widget(&mut self, _widget: &TextFieldWidget) {}
    async fn password_field_widget(&mut self, _widget: &PasswordFieldWidget) {}
    async fn checkbox_widget(&mut self, _widget: &CheckboxWidget) {}
    async fn date_picker_widget(&mut self, _widget: &DatePickerWidget) {}
    async fn select_item_widget(&mut self, _widget: &SelectItemWidget) {}
    async fn select_widget(&mut self, widget: &SelectWidget) {
        for members in &widget.content.ordered_members {
            match members {
                SelectWidgetOrderedMembers::SelectItem(widget) => self.select_item_widget(widget).await
            }
        }
    }
    async fn separator_widget(&mut self, _widget: &SeparatorWidget) {}
    async fn form_widget(&mut self, widget: &FormWidget) {
        if let Some(widget) = &widget.content.actions {
            self.action_panel_widget(widget).await
        }
        for members in &widget.content.ordered_members {
            match members {
                FormWidgetOrderedMembers::TextField(widget) => self.text_field_widget(widget).await,
                FormWidgetOrderedMembers::PasswordField(widget) => self.password_field_widget(widget).await,
                FormWidgetOrderedMembers::Checkbox(widget) => self.checkbox_widget(widget).await,
                FormWidgetOrderedMembers::DatePicker(widget) => self.date_picker_widget(widget).await,
                FormWidgetOrderedMembers::Select(widget) => self.select_widget(widget).await,
                FormWidgetOrderedMembers::Separator(widget) => self.separator_widget(widget).await,
            }
        }
    }

    async fn inline_separator_widget(&mut self, _widget: &InlineSeparatorWidget) {}

    async fn inline_widget(&mut self, widget: &InlineWidget) {
        if let Some(widget) = &widget.content.actions {
            self.action_panel_widget(widget).await
        }
        for members in &widget.content.ordered_members {
            match members {
                InlineWidgetOrderedMembers::Content(widget) => self.content_widget(widget).await,
                InlineWidgetOrderedMembers::InlineSeparator(widget) => self.inline_separator_widget(widget).await
            }
        }
    }

    async fn empty_view_widget(&mut self, widget: &EmptyViewWidget) {
        if let Some(image) = &widget.image {
            self.image(widget.__id__, image).await
        }
    }

    async fn icon_accessory_widget(&mut self, widget: &IconAccessoryWidget) {
        self.image(widget.__id__, &widget.icon).await
    }
    async fn text_accessory_widget(&mut self, widget: &TextAccessoryWidget) {
        if let Some(image) = &widget.icon {
            self.image(widget.__id__, image).await
        }
    }

    async fn search_bar_widget(&mut self, _widget: &SearchBarWidget) {}

    async fn list_item_widget(&mut self, widget: &ListItemWidget) {
        if let Some(image) = &widget.icon {
            self.image(widget.__id__, image).await
        }

        for accessories in &widget.content.accessories {
            match accessories {
                ListItemAccessories::_0(widget) => self.text_accessory_widget(widget).await,
                ListItemAccessories::_1(widget) => self.icon_accessory_widget(widget).await
            }
        }
    }
    async fn list_section_widget(&mut self, widget: &ListSectionWidget) {
        for members in &widget.content.ordered_members {
            match members {
                ListSectionWidgetOrderedMembers::ListItem(widget) => self.list_item_widget(widget).await
            }
        }
    }

    async fn list_widget(&mut self, widget: &ListWidget) {
        if let Some(widget) = &widget.content.actions {
            self.action_panel_widget(widget).await
        }
        if let Some(widget) = &widget.content.search_bar {
            self.search_bar_widget(widget).await
        }
        if let Some(widget) = &widget.content.empty_view {
            self.empty_view_widget(widget).await
        }
        if let Some(widget) = &widget.content.detail {
            self.detail_widget(widget).await
        }
        for members in &widget.content.ordered_members {
            match members {
                ListWidgetOrderedMembers::ListItem(widget) => self.list_item_widget(widget).await,
                ListWidgetOrderedMembers::ListSection(widget) => self.list_section_widget(widget).await,
            }
        }
    }
    async fn grid_item_widget(&mut self, widget: &GridItemWidget) {
        if let Some(widget) = &widget.content.accessory {
            self.icon_accessory_widget(widget).await
        }
        for members in &widget.content.content.content.ordered_members {
            match members {
                ContentWidgetOrderedMembers::Paragraph(widget) => self.paragraph_widget(widget).await,
                ContentWidgetOrderedMembers::Image(widget) => self.image_widget(widget).await,
                ContentWidgetOrderedMembers::H1(widget) => self.h1_widget(widget).await,
                ContentWidgetOrderedMembers::H2(widget) => self.h2_widget(widget).await,
                ContentWidgetOrderedMembers::H3(widget) => self.h3_widget(widget).await,
                ContentWidgetOrderedMembers::H4(widget) => self.h4_widget(widget).await,
                ContentWidgetOrderedMembers::H5(widget) => self.h5_widget(widget).await,
                ContentWidgetOrderedMembers::H6(widget) => self.h6_widget(widget).await,
                ContentWidgetOrderedMembers::HorizontalBreak(widget) => self.horizontal_break_widget(widget).await,
                ContentWidgetOrderedMembers::CodeBlock(widget) => self.code_block_widget(widget).await,
            }
        }
    }
    async fn grid_section_widget(&mut self, widget: &GridSectionWidget) {
        for members in &widget.content.ordered_members {
            match members {
                GridSectionWidgetOrderedMembers::GridItem(widget) => self.grid_item_widget(widget).await
            }
        }
    }
    async fn grid_widget(&mut self, widget: &GridWidget) {
        if let Some(widget) = &widget.content.actions {
            self.action_panel_widget(widget).await
        }
        if let Some(widget) = &widget.content.search_bar {
            self.search_bar_widget(widget).await
        }
        if let Some(widget) = &widget.content.empty_view {
            self.empty_view_widget(widget).await
        }
        for members in &widget.content.ordered_members {
            match members {
                GridWidgetOrderedMembers::GridItem(widget) => self.grid_item_widget(widget).await,
                GridWidgetOrderedMembers::GridSection(widget) => self.grid_section_widget(widget).await
            }
        }
    }

    async fn root_widget(&mut self, root_widget: &RootWidget) {
        if let Some(members) = &root_widget.content {
            match members {
                RootWidgetMembers::Detail(widget) => self.detail_widget(widget).await,
                RootWidgetMembers::Form(widget) => self.form_widget(widget).await,
                RootWidgetMembers::Inline(widget) => self.inline_widget(widget).await,
                RootWidgetMembers::List(widget) => self.list_widget(widget).await,
                RootWidgetMembers::Grid(widget) => self.grid_widget(widget).await,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UiPropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    Bytes(bytes::Bytes),
    Array(Vec<UiPropertyValue>),
    Object(HashMap<String, UiPropertyValue>),
    Undefined,
}

pub type UiWidgetId = usize;

#[derive(Debug, Clone)]
pub struct SettingsEntrypoint {
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub entrypoint_description: String,
    pub entrypoint_type: SettingsEntrypointType,
    pub enabled: bool,
    pub preferences: HashMap<String, PluginPreference>,
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Clone)]
pub struct SettingsPlugin {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub plugin_description: String,
    pub enabled: bool,
    pub entrypoints: HashMap<EntrypointId, SettingsEntrypoint>,
    pub preferences: HashMap<String, PluginPreference>,
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Clone)]
pub enum SettingsEntrypointType {
    Command,
    View,
    InlineView,
    EntrypointGenerator,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SettingsTheme {
    AutoDetect,
    ThemeFile,
    Config,
    // Custom, TODO specify file path or drag and drop via settings ui
    MacOSLight,
    MacOSDark,
    Legacy
}

impl Display for SettingsTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            SettingsTheme::AutoDetect => "Auto-detect",
            SettingsTheme::ThemeFile => "Theme file present",
            SettingsTheme::Config => "Config setting present",
            SettingsTheme::MacOSLight => "macOS Light",
            SettingsTheme::MacOSDark => "macOS Dark",
            SettingsTheme::Legacy => "Legacy",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug, Clone)]
pub enum PluginPreferenceUserData {
    Number {
        value: Option<f64>,
    },
    String {
        value: Option<String>,
    },
    Enum {
        value: Option<String>,
    },
    Bool {
        value: Option<bool>,
    },
    ListOfStrings {
        value: Option<Vec<String>>,
    },
    ListOfNumbers {
        value: Option<Vec<f64>>,
    },
    ListOfEnums {
        value: Option<Vec<String>>,
    },
    // TODO be careful about exposing secrets to logs when adding password type
}

#[derive(Debug, Clone)]
pub enum PluginPreference {
    Number {
        name: String,
        default: Option<f64>,
        description: String,
    },
    String {
        name: String,
        default: Option<String>,
        description: String,
    },
    Enum {
        name: String,
        default: Option<String>,
        description: String,
        enum_values: Vec<PreferenceEnumValue>,
    },
    Bool {
        name: String,
        default: Option<bool>,
        description: String,
    },
    ListOfStrings {
        name: String,
        default: Option<Vec<String>>,
        description: String,
    },
    ListOfNumbers {
        name: String,
        default: Option<Vec<f64>>,
        description: String,
    },
    ListOfEnums {
        name: String,
        default: Option<Vec<String>>,
        enum_values: Vec<PreferenceEnumValue>,
        description: String,
    },
}

#[derive(Debug, Clone)]
pub struct PreferenceEnumValue {
    pub label: String,
    pub value: String,
}


// copy of iced (currently fork) PhysicalKey but without modifiers
#[derive(Debug, Clone)]
pub enum PhysicalKey {
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    Backspace,
    CapsLock,
    ContextMenu,
    Enter,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Abort,
    Resume,
    Suspend,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}

impl PhysicalKey {
    pub fn from_value(key: String) -> PhysicalKey {
        match key.as_str() {
            "Backquote" => PhysicalKey::Backquote,
            "Backslash" => PhysicalKey::Backslash,
            "BracketLeft" => PhysicalKey::BracketLeft,
            "BracketRight" => PhysicalKey::BracketRight,
            "Comma" => PhysicalKey::Comma,
            "Digit0" => PhysicalKey::Digit0,
            "Digit1" => PhysicalKey::Digit1,
            "Digit2" => PhysicalKey::Digit2,
            "Digit3" => PhysicalKey::Digit3,
            "Digit4" => PhysicalKey::Digit4,
            "Digit5" => PhysicalKey::Digit5,
            "Digit6" => PhysicalKey::Digit6,
            "Digit7" => PhysicalKey::Digit7,
            "Digit8" => PhysicalKey::Digit8,
            "Digit9" => PhysicalKey::Digit9,
            "Equal" => PhysicalKey::Equal,
            "IntlBackslash" => PhysicalKey::IntlBackslash,
            "IntlRo" => PhysicalKey::IntlRo,
            "IntlYen" => PhysicalKey::IntlYen,
            "KeyA" => PhysicalKey::KeyA,
            "KeyB" => PhysicalKey::KeyB,
            "KeyC" => PhysicalKey::KeyC,
            "KeyD" => PhysicalKey::KeyD,
            "KeyE" => PhysicalKey::KeyE,
            "KeyF" => PhysicalKey::KeyF,
            "KeyG" => PhysicalKey::KeyG,
            "KeyH" => PhysicalKey::KeyH,
            "KeyI" => PhysicalKey::KeyI,
            "KeyJ" => PhysicalKey::KeyJ,
            "KeyK" => PhysicalKey::KeyK,
            "KeyL" => PhysicalKey::KeyL,
            "KeyM" => PhysicalKey::KeyM,
            "KeyN" => PhysicalKey::KeyN,
            "KeyO" => PhysicalKey::KeyO,
            "KeyP" => PhysicalKey::KeyP,
            "KeyQ" => PhysicalKey::KeyQ,
            "KeyR" => PhysicalKey::KeyR,
            "KeyS" => PhysicalKey::KeyS,
            "KeyT" => PhysicalKey::KeyT,
            "KeyU" => PhysicalKey::KeyU,
            "KeyV" => PhysicalKey::KeyV,
            "KeyW" => PhysicalKey::KeyW,
            "KeyX" => PhysicalKey::KeyX,
            "KeyY" => PhysicalKey::KeyY,
            "KeyZ" => PhysicalKey::KeyZ,
            "Minus" => PhysicalKey::Minus,
            "Period" => PhysicalKey::Period,
            "Quote" => PhysicalKey::Quote,
            "Semicolon" => PhysicalKey::Semicolon,
            "Slash" => PhysicalKey::Slash,
            "Backspace" => PhysicalKey::Backspace,
            "CapsLock" => PhysicalKey::CapsLock,
            "ContextMenu" => PhysicalKey::ContextMenu,
            "Enter" => PhysicalKey::Enter,
            "Space" => PhysicalKey::Space,
            "Tab" => PhysicalKey::Tab,
            "Convert" => PhysicalKey::Convert,
            "KanaMode" => PhysicalKey::KanaMode,
            "Lang1" => PhysicalKey::Lang1,
            "Lang2" => PhysicalKey::Lang2,
            "Lang3" => PhysicalKey::Lang3,
            "Lang4" => PhysicalKey::Lang4,
            "Lang5" => PhysicalKey::Lang5,
            "NonConvert" => PhysicalKey::NonConvert,
            "Delete" => PhysicalKey::Delete,
            "End" => PhysicalKey::End,
            "Help" => PhysicalKey::Help,
            "Home" => PhysicalKey::Home,
            "Insert" => PhysicalKey::Insert,
            "PageDown" => PhysicalKey::PageDown,
            "PageUp" => PhysicalKey::PageUp,
            "ArrowDown" => PhysicalKey::ArrowDown,
            "ArrowLeft" => PhysicalKey::ArrowLeft,
            "ArrowRight" => PhysicalKey::ArrowRight,
            "ArrowUp" => PhysicalKey::ArrowUp,
            "NumLock" => PhysicalKey::NumLock,
            "Numpad0" => PhysicalKey::Numpad0,
            "Numpad1" => PhysicalKey::Numpad1,
            "Numpad2" => PhysicalKey::Numpad2,
            "Numpad3" => PhysicalKey::Numpad3,
            "Numpad4" => PhysicalKey::Numpad4,
            "Numpad5" => PhysicalKey::Numpad5,
            "Numpad6" => PhysicalKey::Numpad6,
            "Numpad7" => PhysicalKey::Numpad7,
            "Numpad8" => PhysicalKey::Numpad8,
            "Numpad9" => PhysicalKey::Numpad9,
            "NumpadAdd" => PhysicalKey::NumpadAdd,
            "NumpadBackspace" => PhysicalKey::NumpadBackspace,
            "NumpadClear" => PhysicalKey::NumpadClear,
            "NumpadClearEntry" => PhysicalKey::NumpadClearEntry,
            "NumpadComma" => PhysicalKey::NumpadComma,
            "NumpadDecimal" => PhysicalKey::NumpadDecimal,
            "NumpadDivide" => PhysicalKey::NumpadDivide,
            "NumpadEnter" => PhysicalKey::NumpadEnter,
            "NumpadEqual" => PhysicalKey::NumpadEqual,
            "NumpadHash" => PhysicalKey::NumpadHash,
            "NumpadMemoryAdd" => PhysicalKey::NumpadMemoryAdd,
            "NumpadMemoryClear" => PhysicalKey::NumpadMemoryClear,
            "NumpadMemoryRecall" => PhysicalKey::NumpadMemoryRecall,
            "NumpadMemoryStore" => PhysicalKey::NumpadMemoryStore,
            "NumpadMemorySubtract" => PhysicalKey::NumpadMemorySubtract,
            "NumpadMultiply" => PhysicalKey::NumpadMultiply,
            "NumpadParenLeft" => PhysicalKey::NumpadParenLeft,
            "NumpadParenRight" => PhysicalKey::NumpadParenRight,
            "NumpadStar" => PhysicalKey::NumpadStar,
            "NumpadSubtract" => PhysicalKey::NumpadSubtract,
            "Escape" => PhysicalKey::Escape,
            "Fn" => PhysicalKey::Fn,
            "FnLock" => PhysicalKey::FnLock,
            "PrintScreen" => PhysicalKey::PrintScreen,
            "ScrollLock" => PhysicalKey::ScrollLock,
            "Pause" => PhysicalKey::Pause,
            "BrowserBack" => PhysicalKey::BrowserBack,
            "BrowserFavorites" => PhysicalKey::BrowserFavorites,
            "BrowserForward" => PhysicalKey::BrowserForward,
            "BrowserHome" => PhysicalKey::BrowserHome,
            "BrowserRefresh" => PhysicalKey::BrowserRefresh,
            "BrowserSearch" => PhysicalKey::BrowserSearch,
            "BrowserStop" => PhysicalKey::BrowserStop,
            "Eject" => PhysicalKey::Eject,
            "LaunchApp1" => PhysicalKey::LaunchApp1,
            "LaunchApp2" => PhysicalKey::LaunchApp2,
            "LaunchMail" => PhysicalKey::LaunchMail,
            "MediaPlayPause" => PhysicalKey::MediaPlayPause,
            "MediaSelect" => PhysicalKey::MediaSelect,
            "MediaStop" => PhysicalKey::MediaStop,
            "MediaTrackNext" => PhysicalKey::MediaTrackNext,
            "MediaTrackPrevious" => PhysicalKey::MediaTrackPrevious,
            "Power" => PhysicalKey::Power,
            "Sleep" => PhysicalKey::Sleep,
            "AudioVolumeDown" => PhysicalKey::AudioVolumeDown,
            "AudioVolumeMute" => PhysicalKey::AudioVolumeMute,
            "AudioVolumeUp" => PhysicalKey::AudioVolumeUp,
            "WakeUp" => PhysicalKey::WakeUp,
            "Abort" => PhysicalKey::Abort,
            "Resume" => PhysicalKey::Resume,
            "Suspend" => PhysicalKey::Suspend,
            "Again" => PhysicalKey::Again,
            "Copy" => PhysicalKey::Copy,
            "Cut" => PhysicalKey::Cut,
            "Find" => PhysicalKey::Find,
            "Open" => PhysicalKey::Open,
            "Paste" => PhysicalKey::Paste,
            "Props" => PhysicalKey::Props,
            "Select" => PhysicalKey::Select,
            "Undo" => PhysicalKey::Undo,
            "Hiragana" => PhysicalKey::Hiragana,
            "Katakana" => PhysicalKey::Katakana,
            "F1" => PhysicalKey::F1,
            "F2" => PhysicalKey::F2,
            "F3" => PhysicalKey::F3,
            "F4" => PhysicalKey::F4,
            "F5" => PhysicalKey::F5,
            "F6" => PhysicalKey::F6,
            "F7" => PhysicalKey::F7,
            "F8" => PhysicalKey::F8,
            "F9" => PhysicalKey::F9,
            "F10" => PhysicalKey::F10,
            "F11" => PhysicalKey::F11,
            "F12" => PhysicalKey::F12,
            "F13" => PhysicalKey::F13,
            "F14" => PhysicalKey::F14,
            "F15" => PhysicalKey::F15,
            "F16" => PhysicalKey::F16,
            "F17" => PhysicalKey::F17,
            "F18" => PhysicalKey::F18,
            "F19" => PhysicalKey::F19,
            "F20" => PhysicalKey::F20,
            "F21" => PhysicalKey::F21,
            "F22" => PhysicalKey::F22,
            "F23" => PhysicalKey::F23,
            "F24" => PhysicalKey::F24,
            "F25" => PhysicalKey::F25,
            "F26" => PhysicalKey::F26,
            "F27" => PhysicalKey::F27,
            "F28" => PhysicalKey::F28,
            "F29" => PhysicalKey::F29,
            "F30" => PhysicalKey::F30,
            "F31" => PhysicalKey::F31,
            "F32" => PhysicalKey::F32,
            "F33" => PhysicalKey::F33,
            "F34" => PhysicalKey::F34,
            "F35" => PhysicalKey::F35,
            _ => {
                panic!("unknown key: {}", key)
            }
        }
    }

    pub fn to_value(&self) -> String {
        match self {
            PhysicalKey::Backquote => "Backquote",
            PhysicalKey::Backslash => "Backslash",
            PhysicalKey::BracketLeft => "BracketLeft",
            PhysicalKey::BracketRight => "BracketRight",
            PhysicalKey::Comma => "Comma",
            PhysicalKey::Digit0 => "Digit0",
            PhysicalKey::Digit1 => "Digit1",
            PhysicalKey::Digit2 => "Digit2",
            PhysicalKey::Digit3 => "Digit3",
            PhysicalKey::Digit4 => "Digit4",
            PhysicalKey::Digit5 => "Digit5",
            PhysicalKey::Digit6 => "Digit6",
            PhysicalKey::Digit7 => "Digit7",
            PhysicalKey::Digit8 => "Digit8",
            PhysicalKey::Digit9 => "Digit9",
            PhysicalKey::Equal => "Equal",
            PhysicalKey::IntlBackslash => "IntlBackslash",
            PhysicalKey::IntlRo => "IntlRo",
            PhysicalKey::IntlYen => "IntlYen",
            PhysicalKey::KeyA => "KeyA",
            PhysicalKey::KeyB => "KeyB",
            PhysicalKey::KeyC => "KeyC",
            PhysicalKey::KeyD => "KeyD",
            PhysicalKey::KeyE => "KeyE",
            PhysicalKey::KeyF => "KeyF",
            PhysicalKey::KeyG => "KeyG",
            PhysicalKey::KeyH => "KeyH",
            PhysicalKey::KeyI => "KeyI",
            PhysicalKey::KeyJ => "KeyJ",
            PhysicalKey::KeyK => "KeyK",
            PhysicalKey::KeyL => "KeyL",
            PhysicalKey::KeyM => "KeyM",
            PhysicalKey::KeyN => "KeyN",
            PhysicalKey::KeyO => "KeyO",
            PhysicalKey::KeyP => "KeyP",
            PhysicalKey::KeyQ => "KeyQ",
            PhysicalKey::KeyR => "KeyR",
            PhysicalKey::KeyS => "KeyS",
            PhysicalKey::KeyT => "KeyT",
            PhysicalKey::KeyU => "KeyU",
            PhysicalKey::KeyV => "KeyV",
            PhysicalKey::KeyW => "KeyW",
            PhysicalKey::KeyX => "KeyX",
            PhysicalKey::KeyY => "KeyY",
            PhysicalKey::KeyZ => "KeyZ",
            PhysicalKey::Minus => "Minus",
            PhysicalKey::Period => "Period",
            PhysicalKey::Quote => "Quote",
            PhysicalKey::Semicolon => "Semicolon",
            PhysicalKey::Slash => "Slash",
            PhysicalKey::Backspace => "Backspace",
            PhysicalKey::CapsLock => "CapsLock",
            PhysicalKey::ContextMenu => "ContextMenu",
            PhysicalKey::Enter => "Enter",
            PhysicalKey::Space => "Space",
            PhysicalKey::Tab => "Tab",
            PhysicalKey::Convert => "Convert",
            PhysicalKey::KanaMode => "KanaMode",
            PhysicalKey::Lang1 => "Lang1",
            PhysicalKey::Lang2 => "Lang2",
            PhysicalKey::Lang3 => "Lang3",
            PhysicalKey::Lang4 => "Lang4",
            PhysicalKey::Lang5 => "Lang5",
            PhysicalKey::NonConvert => "NonConvert",
            PhysicalKey::Delete => "Delete",
            PhysicalKey::End => "End",
            PhysicalKey::Help => "Help",
            PhysicalKey::Home => "Home",
            PhysicalKey::Insert => "Insert",
            PhysicalKey::PageDown => "PageDown",
            PhysicalKey::PageUp => "PageUp",
            PhysicalKey::ArrowDown => "ArrowDown",
            PhysicalKey::ArrowLeft => "ArrowLeft",
            PhysicalKey::ArrowRight => "ArrowRight",
            PhysicalKey::ArrowUp => "ArrowUp",
            PhysicalKey::NumLock => "NumLock",
            PhysicalKey::Numpad0 => "Numpad0",
            PhysicalKey::Numpad1 => "Numpad1",
            PhysicalKey::Numpad2 => "Numpad2",
            PhysicalKey::Numpad3 => "Numpad3",
            PhysicalKey::Numpad4 => "Numpad4",
            PhysicalKey::Numpad5 => "Numpad5",
            PhysicalKey::Numpad6 => "Numpad6",
            PhysicalKey::Numpad7 => "Numpad7",
            PhysicalKey::Numpad8 => "Numpad8",
            PhysicalKey::Numpad9 => "Numpad9",
            PhysicalKey::NumpadAdd => "NumpadAdd",
            PhysicalKey::NumpadBackspace => "NumpadBackspace",
            PhysicalKey::NumpadClear => "NumpadClear",
            PhysicalKey::NumpadClearEntry => "NumpadClearEntry",
            PhysicalKey::NumpadComma => "NumpadComma",
            PhysicalKey::NumpadDecimal => "NumpadDecimal",
            PhysicalKey::NumpadDivide => "NumpadDivide",
            PhysicalKey::NumpadEnter => "NumpadEnter",
            PhysicalKey::NumpadEqual => "NumpadEqual",
            PhysicalKey::NumpadHash => "NumpadHash",
            PhysicalKey::NumpadMemoryAdd => "NumpadMemoryAdd",
            PhysicalKey::NumpadMemoryClear => "NumpadMemoryClear",
            PhysicalKey::NumpadMemoryRecall => "NumpadMemoryRecall",
            PhysicalKey::NumpadMemoryStore => "NumpadMemoryStore",
            PhysicalKey::NumpadMemorySubtract => "NumpadMemorySubtract",
            PhysicalKey::NumpadMultiply => "NumpadMultiply",
            PhysicalKey::NumpadParenLeft => "NumpadParenLeft",
            PhysicalKey::NumpadParenRight => "NumpadParenRight",
            PhysicalKey::NumpadStar => "NumpadStar",
            PhysicalKey::NumpadSubtract => "NumpadSubtract",
            PhysicalKey::Escape => "Escape",
            PhysicalKey::Fn => "Fn",
            PhysicalKey::FnLock => "FnLock",
            PhysicalKey::PrintScreen => "PrintScreen",
            PhysicalKey::ScrollLock => "ScrollLock",
            PhysicalKey::Pause => "Pause",
            PhysicalKey::BrowserBack => "BrowserBack",
            PhysicalKey::BrowserFavorites => "BrowserFavorites",
            PhysicalKey::BrowserForward => "BrowserForward",
            PhysicalKey::BrowserHome => "BrowserHome",
            PhysicalKey::BrowserRefresh => "BrowserRefresh",
            PhysicalKey::BrowserSearch => "BrowserSearch",
            PhysicalKey::BrowserStop => "BrowserStop",
            PhysicalKey::Eject => "Eject",
            PhysicalKey::LaunchApp1 => "LaunchApp1",
            PhysicalKey::LaunchApp2 => "LaunchApp2",
            PhysicalKey::LaunchMail => "LaunchMail",
            PhysicalKey::MediaPlayPause => "MediaPlayPause",
            PhysicalKey::MediaSelect => "MediaSelect",
            PhysicalKey::MediaStop => "MediaStop",
            PhysicalKey::MediaTrackNext => "MediaTrackNext",
            PhysicalKey::MediaTrackPrevious => "MediaTrackPrevious",
            PhysicalKey::Power => "Power",
            PhysicalKey::Sleep => "Sleep",
            PhysicalKey::AudioVolumeDown => "AudioVolumeDown",
            PhysicalKey::AudioVolumeMute => "AudioVolumeMute",
            PhysicalKey::AudioVolumeUp => "AudioVolumeUp",
            PhysicalKey::WakeUp => "WakeUp",
            PhysicalKey::Abort => "Abort",
            PhysicalKey::Resume => "Resume",
            PhysicalKey::Suspend => "Suspend",
            PhysicalKey::Again => "Again",
            PhysicalKey::Copy => "Copy",
            PhysicalKey::Cut => "Cut",
            PhysicalKey::Find => "Find",
            PhysicalKey::Open => "Open",
            PhysicalKey::Paste => "Paste",
            PhysicalKey::Props => "Props",
            PhysicalKey::Select => "Select",
            PhysicalKey::Undo => "Undo",
            PhysicalKey::Hiragana => "Hiragana",
            PhysicalKey::Katakana => "Katakana",
            PhysicalKey::F1 => "F1",
            PhysicalKey::F2 => "F2",
            PhysicalKey::F3 => "F3",
            PhysicalKey::F4 => "F4",
            PhysicalKey::F5 => "F5",
            PhysicalKey::F6 => "F6",
            PhysicalKey::F7 => "F7",
            PhysicalKey::F8 => "F8",
            PhysicalKey::F9 => "F9",
            PhysicalKey::F10 => "F10",
            PhysicalKey::F11 => "F11",
            PhysicalKey::F12 => "F12",
            PhysicalKey::F13 => "F13",
            PhysicalKey::F14 => "F14",
            PhysicalKey::F15 => "F15",
            PhysicalKey::F16 => "F16",
            PhysicalKey::F17 => "F17",
            PhysicalKey::F18 => "F18",
            PhysicalKey::F19 => "F19",
            PhysicalKey::F20 => "F20",
            PhysicalKey::F21 => "F21",
            PhysicalKey::F22 => "F22",
            PhysicalKey::F23 => "F23",
            PhysicalKey::F24 => "F24",
            PhysicalKey::F25 => "F25",
            PhysicalKey::F26 => "F26",
            PhysicalKey::F27 => "F27",
            PhysicalKey::F28 => "F28",
            PhysicalKey::F29 => "F29",
            PhysicalKey::F30 => "F30",
            PhysicalKey::F31 => "F31",
            PhysicalKey::F32 => "F32",
            PhysicalKey::F33 => "F33",
            PhysicalKey::F34 => "F34",
            PhysicalKey::F35 => "F35",
        }.to_string()
    }
}

