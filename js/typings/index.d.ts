
// js runtime types

type DesktopPathAction<DATA> = DesktopPathActionAdd<DATA> | DesktopPathActionRemove

type DesktopPathActionAdd<DATA> = {
    type: "add",
    id: string,
    data: DATA
}

type DesktopPathActionRemove = {
    type: "remove"
    id: string
}

type LinuxDesktopApplicationData = {
    name: string
    icon: ArrayBuffer | undefined,
    startup_wm_class: string | undefined,
}

type MacOSDesktopApplicationData = {
    name: string
    path: string,
    icon: ArrayBuffer | undefined,
}

type MacOSDesktopSettingsPre13Data = {
    name: string
    path: string,
    icon: ArrayBuffer | undefined,
}

type MacOSDesktopSettings13AndPostData = {
    name: string
    preferences_id: string
    icon: ArrayBuffer | undefined,
}

type PluginEvent = ViewEvent | NotReactsKeyboardEvent | RunCommand | RunGeneratedCommand | OpenView | CloseView | OpenInlineView | ReloadSearchIndex | RefreshSearchIndex
type RenderLocation = "InlineView" | "View"

type ViewEvent = {
    type: "ViewEvent"
    widgetId: number
    eventName: string
    eventArguments: PropertyValue[]
}

type KeyboardEventOrigin = "MainView" | "PluginView"

// naming to avoid collision
type NotReactsKeyboardEvent = {
    type: "KeyboardEvent"
    entrypointId: string
    origin: KeyboardEventOrigin
    key: string
    modifierShift: boolean
    modifierControl: boolean
    modifierAlt: boolean
    modifierMeta: boolean
}

type OpenView = {
    type: "OpenView"
    entrypointId: string
}

type CloseView = {
    type: "CloseView"
}

type RunCommand = {
    type: "RunCommand"
    entrypointId: string
}

type RunGeneratedCommand = {
    type: "RunGeneratedCommand"
    entrypointId: string
    actionIndex: number
}

type OpenInlineView = {
    type: "OpenInlineView"
    text: string
}

type ReloadSearchIndex = {
    type: "ReloadSearchIndex"
}

type RefreshSearchIndex = {
    type: "RefreshSearchIndex"
}

type PropertyValue = PropertyValueString | PropertyValueNumber | PropertyValueBool | PropertyValueUndefined
type PropertyValueString = { type: "String", value: string }
type PropertyValueNumber = { type: "Number", value: number }
type PropertyValueBool = { type: "Bool", value: boolean }
type PropertyValueUndefined = { type: "Undefined" }

type UiWidget = {
    widgetId: number,
    widgetType: string,
    widgetProperties: Props,
    widgetChildren: UiWidget[],
}

type Props = { [key: string]: any };
type PropsWithChildren = { children?: UiWidget[] } & Props;

type GeneratedCommandAccessory = GeneratedCommandTextAccessory | GeneratedCommandIconAccessory;

interface GeneratedCommandTextAccessory {
    text: string
    icon?: string
    tooltip?: string
}

interface GeneratedCommandIconAccessory {
    icon: string
    tooltip?: string
}

type GeneratedSearchItem = {
    entrypoint_name: string,
    entrypoint_id: string,
    entrypoint_uuid: string,
    entrypoint_icon: ArrayBuffer | undefined,
    entrypoint_actions: GeneratedSearchItemAction[],
    entrypoint_accessories: GeneratedCommandAccessory[],
}

type GeneratedSearchItemAction = {
    id?: string,
    action_type: "Command" | "View"
    label: string,
}

declare module "gauntlet:bridge/internal-all" {
    function open_settings(): void
    function run_numbat(input: string): { left: string, right: string }
    function current_os(): string
    function wayland(): boolean
    function application_pending_event(): Promise<object>
}

declare module "gauntlet:bridge/internal-linux" {
    function linux_open_application(desktop_id: string): void
    function linux_application_dirs(): string[]
    function linux_app_from_path(path: string): Promise<undefined | DesktopPathAction<LinuxDesktopApplicationData>>

}

declare module "gauntlet:bridge/internal-macos" {
    function macos_major_version(): number
    function macos_settings_pre_13(): MacOSDesktopSettingsPre13Data[]
    function macos_settings_13_and_post(): MacOSDesktopSettings13AndPostData[]
    function macos_open_setting_13_and_post(preferences_id: String): void
    function macos_open_setting_pre_13(setting_path: String): void

    function macos_system_applications(): string[]
    function macos_application_dirs(): string[]
    function macos_app_from_path(path: string): Promise<undefined | DesktopPathAction<MacOSDesktopApplicationData>>
    function macos_app_from_arbitrary_path(path: string): Promise<undefined | DesktopPathAction<MacOSDesktopApplicationData>>
    function macos_open_application(app_path: String): void
}

declare module "ext:core/ops" {
    function open_settings(): void
    function run_numbat(input: string): { left: string, right: string }

    function current_os(): string
    function wayland(): boolean
    function application_pending_event(): Promise<object>

    function linux_open_application(desktop_id: string): void
    function linux_application_dirs(): string[]
    function linux_app_from_path(path: string): Promise<undefined | DesktopPathAction<LinuxDesktopApplicationData>>

    function macos_major_version(): number
    function macos_settings_pre_13(): MacOSDesktopSettingsPre13Data[]
    function macos_settings_13_and_post(): MacOSDesktopSettings13AndPostData[]
    function macos_open_setting_13_and_post(preferences_id: String): void
    function macos_open_setting_pre_13(setting_path: String): void

    function macos_system_applications(): string[]
    function macos_application_dirs(): string[]
    function macos_app_from_path(path: string): Promise<undefined | DesktopPathAction<MacOSDesktopApplicationData>>
    function macos_app_from_arbitrary_path(path: string): Promise<undefined | DesktopPathAction<MacOSDesktopApplicationData>>
    function macos_open_application(app_path: String): void

    function op_log_trace(target: string, message: string): void;
    function op_log_debug(target: string, message: string): void;
    function op_log_info(target: string, message: string): void;
    function op_log_warn(target: string, message: string): void;
    function op_log_error(target: string, message: string): void;

    function op_component_model(): Record<string, Component>;
    function asset_data(path: string): Promise<number[]>;
    function asset_data_blocking(path: string): number[];

    function op_inline_view_entrypoint_id(): string | null;
    function op_entrypoint_names(): Record<string, string | undefined>;
    function clear_inline_view(): void;
    function op_plugin_get_pending_event(): Promise<PluginEvent>;

    function get_entrypoint_generator_entrypoint_ids(): Promise<string[]>

    function get_plugin_preferences(): Record<string, any>;
    function get_entrypoint_preferences(entrypointId: string): Record<string, any>;
    function plugin_preferences_required(): Promise<boolean>;
    function entrypoint_preferences_required(entrypointId: string): Promise<boolean>;
    function show_preferences_required_view(entrypointId: string, pluginPreferencesRequired: boolean, entrypointPreferencesRequired: boolean): void;

    function reload_search_index(searchItems: GeneratedSearchItem[], refreshSearchList: boolean): Promise<void>;

    function show_hud(display: string): void;
    function update_loading_bar(entrypoint_id: string, show: boolean): void;

    function op_react_replace_view(render_location: RenderLocation, top_level_view: boolean, entrypoint_id: string, entrypoint_name: string, container: any): void;
    function show_plugin_error_view(entrypoint_id: string, render_location: RenderLocation): void;

    function fetch_action_id_for_shortcut(entrypointId: string, key: string, modifierShift: boolean, modifierControl: boolean, modifierAlt: boolean, modifierMeta: boolean): Promise<string | undefined>;

    function clipboard_read(): Promise<{ text_data?: string, png_data?: number[] }>;
    function clipboard_read_text(): Promise<string | undefined>;
    function clipboard_write(data: { text_data?: string, png_data?: number[] }): Promise<void>;
    function clipboard_write_text(data: string): Promise<void>;
    function clipboard_clear(): Promise<void>;

    function environment_gauntlet_version(): number;
    function environment_is_development(): boolean;
    function environment_plugin_data_dir(): string;
    function environment_plugin_cache_dir(): string;
}

// component model types

type Component = StandardComponent | RootComponent | TextPartComponent

type StandardComponent = {
    type: "standard",
    internalName: string,
    name: string,
    props: Property[],
    children: Children,
}

type RootComponent = {
    type: "root",
    internalName: string,
    children: ComponentRef[],
    sharedTypes: Record<string, SharedType>
}

type SharedType = SharedTypeEnum | SharedTypeObject | SharedTypeUnion

type SharedTypeEnum = {
    type: "enum",
    items: string[],
}

type SharedTypeObject = {
    type: "object",
    items: Record<string, PropertyType>
}

type SharedTypeUnion = {
    type: "union",
    items: PropertyType[]
}

type TextPartComponent = {
    type: "text_part",
    internalName: string,
}

type Property = {
    name: string
    optional: boolean
    type: PropertyType
}
type Children = ChildrenMembers | ChildrenString | ChildrenNone | ChildrenStringOrMembers

type ChildrenMembers = {
    type: "members",
    ordered_members: Record<string, ComponentRef>
    per_type_members: Record<string, ComponentRef>
}
type ChildrenStringOrMembers = {
    type: "string_or_members",
    textPartInternalName: string,
    ordered_members: Record<string, ComponentRef>
    per_type_members: Record<string, ComponentRef>
}
type ChildrenString = {
    type: "string"
    textPartInternalName: string,
}
type ChildrenNone = {
    type: "none"
}

type ComponentRef = {
    componentInternalName: string,
    componentName: string,
}

type PropertyType = TypeString | TypeNumber | TypeBoolean | TypeComponent | TypeFunction | TypeSharedTypeRef | TypeImageArray | TypeImageUnion

type TypeString = {
    type: "string"
}
type TypeNumber = {
    type: "number"
}
type TypeBoolean = {
    type: "boolean"
}
type TypeComponent = {
    type: "component"
    reference: ComponentRef,
}
type TypeFunction = {
    type: "function"
    arguments: Property[]
}
type TypeSharedTypeRef = {
    type: "shared_type_ref"
    name: string
}
type TypeImageUnion = {
    type: "union"
    items: PropertyType[]
}
type TypeImageArray = {
    type: "array"
    item: PropertyType
}
