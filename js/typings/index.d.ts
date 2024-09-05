
// js runtime types

interface DenoCore {
    opAsync: (op: "op_plugin_get_pending_event") => Promise<PluginEvent>
    ops: InternalApi
}

type PluginEvent = ViewEvent | NotReactsKeyboardEvent | RunCommand | RunGeneratedCommand | OpenView | CloseView | OpenInlineView | ReloadSearchIndex | RefreshSearchIndex
type RenderLocation = "InlineView" | "View"

type ViewEvent = {
    type: "ViewEvent"
    widgetId: number
    eventName: string
    eventArguments: PropertyValue[]
}

// naming to avoid collision
type NotReactsKeyboardEvent = {
    type: "KeyboardEvent"
    entrypointId: string
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

type AdditionalSearchItem = {
    entrypoint_name: string,
    entrypoint_id: string,
    entrypoint_uuid: string,
    entrypoint_icon: ArrayBuffer | undefined,
}

interface InternalApi {
    op_log_trace(target: string, message: string): void;
    op_log_debug(target: string, message: string): void;
    op_log_info(target: string, message: string): void;
    op_log_warn(target: string, message: string): void;
    op_log_error(target: string, message: string): void;

    op_component_model(): Record<string, Component>;
    asset_data(path: string): Promise<number[]>;
    asset_data_blocking(path: string): number[];

    op_inline_view_endpoint_id(): string | null;
    clear_inline_view(): void;

    get_command_generator_entrypoint_ids(): Promise<string[]>
    get_plugin_preferences(): Record<string, any>;
    get_entrypoint_preferences(entrypointId: string): Record<string, any>;
    plugin_preferences_required(): Promise<boolean>;
    entrypoint_preferences_required(entrypointId: string): Promise<boolean>;
    show_preferences_required_view(entrypointId: string, pluginPreferencesRequired: boolean, entrypointPreferencesRequired: boolean): void;

    reload_search_index(searchItems: AdditionalSearchItem[], refreshSearchList: boolean): Promise<void>;

    op_react_replace_view(render_location: RenderLocation, top_level_view: boolean, entrypoint_id: string, container: UiWidget): void;
    show_plugin_error_view(entrypoint_id: string, render_location: RenderLocation): void;

    fetch_action_id_for_shortcut(entrypointId: string, key: string, modifierShift: boolean, modifierControl: boolean, modifierAlt: boolean, modifierMeta: boolean): Promise<string>;

    clipboard_read(): Promise<{ text_data?: string, png_data?: Blob }>;
    clipboard_read_text(): Promise<string | undefined>;
    clipboard_write(data: { text_data?: string, png_data?: number[] }): Promise<void>;
    clipboard_write_text(data: string): Promise<void>;
    clipboard_clear(): Promise<void>;
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

type SharedType = SharedTypeEnum | SharedTypeObject

type SharedTypeEnum = {
    type: "enum",
    items: string[],
}

type SharedTypeObject = {
    type: "object",
    items: Record<string, PropertyType>
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
    members: Record<string, ComponentRef>
}
type ChildrenStringOrMembers = {
    type: "string_or_members",
    textPartInternalName: string,
    members: Record<string, ComponentRef>
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

type PropertyType = TypeString | TypeNumber | TypeBoolean | TypeComponent | TypeFunction | TypeImageSource | TypeImageEnum | TypeImageUnion | TypeImageObject

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
type TypeImageSource = {
    type: "image_source"
}
type TypeImageEnum = {
    type: "enum"
    name: string
}
type TypeImageUnion = {
    type: "union"
    items: PropertyType[]
}
type TypeImageObject = {
    type: "object"
    name: string
}