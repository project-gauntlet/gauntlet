declare namespace Deno {
    const internal: unique symbol;
    function inspect(value: unknown, options?: InspectOptions): string
}

declare interface InspectOptions {
    depth: number
}

declare interface Deno {
    [Deno.internal]: {
        core: {
            opAsync: (op: string) => Promise<PluginEvent>
            ops: InternalApi
        }
    };
}

declare type PluginEvent = ViewEvent | ViewCreated | ViewDestroyed | PluginCommand

declare type ViewEvent =  {
    type: "ViewEvent"
    eventName: string
    widget: InstanceSync
}

declare type ViewCreated =  {
    type: "ViewCreated"
    reconcilerMode: string
    viewName: string
}

declare type ViewDestroyed =  {
    type: "ViewDestroyed"
}

declare type PluginCommand =  {
    type: "PluginCommand"
    commandType: "stop"
}

declare type UiWidget = {}

declare type Type = string;
declare type Props = { children?: any } & { [key: string]: any };

declare type Container = Instance
declare type Instance = Promise<UiWidget>
declare type TextInstance = Promise<UiWidget>
declare type InstanceSync = UiWidget
declare type TextInstanceSync = UiWidget
declare type ChildSet = (InstanceSync | TextInstanceSync)[]

type SuspenseInstance = never;

declare interface InternalApi {
    op_react_call_event_listener(instance: InstanceSync, eventName: string): void;

    op_react_get_container(): Container;

    op_react_create_instance(type: Type, props: Props): Instance;

    op_react_create_text_instance(text: string): TextInstance;

    op_react_append_child(parent: InstanceSync, child: InstanceSync | TextInstanceSync): void;

    op_react_call_event_listener(instance: InstanceSync, eventName: string): void;

    // mutation mode
    op_react_remove_child(parent: InstanceSync, child: InstanceSync | TextInstanceSync): void;

    op_react_insert_before(
        parent: InstanceSync,
        child: InstanceSync | TextInstanceSync | SuspenseInstance,
        beforeChild: InstanceSync | TextInstanceSync | SuspenseInstance
    ): void;

    op_react_set_properties(instance: InstanceSync, properties: Props): void;

    op_react_set_text(instance: InstanceSync, text: string): void;

    // persistent mode
    op_react_clone_instance(type: Type, properties: Props): Instance;

    op_react_replace_container_children(container: InstanceSync, newChildren: ChildSet): void;
}
