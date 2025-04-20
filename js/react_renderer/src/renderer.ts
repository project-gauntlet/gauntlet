import ReactReconciler, { HostConfig, OpaqueHandle } from "react-reconciler";
import { createContext, ReactNode, useContext } from 'react';
import { DefaultEventPriority } from 'react-reconciler/constants';
import {
    asset_data,
    asset_data_blocking,
    get_entrypoint_preferences,
    get_plugin_preferences,
    op_component_model,
    op_log_trace,
    op_react_replace_view,
    show_hud
} from "ext:core/ops";

// Usage of MessageChannel seems to block Deno runtime from exiting
// causing plugin to be in stuck state where it is disabled but still have running runtime
//
// For some reason React prefers MessageChannel to setTimeout but
// will fall back on setTimeout if MessageChannel is not present
globalThis.MessageChannel = undefined as any;

class HostContext {
    constructor(public nextId: number, public componentModel: Record<string, Component>) {
    }

    [Symbol.for("Deno.customInspect")]() {
        return `-- RootContext --`;
    }
}

type Instance = UiWidget & {
    hostContext: HostContext
}

type RootUiWidget = UiWidget
type PublicInstance = Instance;
type TextInstance = Instance
type TimeoutHandle = number;
type NoTimeout = -1;
type ComponentType = string;
type UpdatePayload = string[];
type SuspenseInstance = never;
type ChildSet = UiWidget[]

class GauntletContextValue {
    private _navStack: ReactNode[] = []
    private _renderLocation: RenderLocation | undefined
    private _rerender: ((node: ReactNode) => void) | undefined
    private _entrypointId: string | undefined;
    private _entrypointName: string | undefined;
    private _clear: (() => void) | undefined;

    reset(entrypointId: string, entrypointName: string, renderLocation: RenderLocation, view: ReactNode, rerender: (node: ReactNode) => void, clear: () => void) {
        this._entrypointId = entrypointId
        this._entrypointName = entrypointName
        this._renderLocation = renderLocation
        this._rerender = rerender
        this._clear = clear
        this._navStack = []
        this._navStack.push(view)
    }

    renderLocation = (): RenderLocation => {
        return this._renderLocation!!
    }

    isBottommostView = () => {
        return this._navStack.length === 1
    }

    topmostView = () => {
        return this._navStack[this._navStack.length - 1]
    }

    entrypointId = () => {
        return this._entrypointId!!
    }

    entrypointName = () => {
        return this._entrypointName!!
    }

    rerender = (component: ReactNode) => {
        this._rerender!!(component)
    };

    clear = () => {
        this._clear!!()
    };

    pushView = (component: ReactNode) => {
        this._navStack.push(component)

        this.rerender(component)
    };

    popView = () => {
        this._navStack.pop();

        this.rerender(this.topmostView())
    };

    entrypointPreferences = () => {
        return get_entrypoint_preferences(this.entrypointId())
    }

    pluginPreferences = () => {
        return get_plugin_preferences()
    }
}

const gauntletContextValue = new GauntletContextValue()
const gauntletContext = createContext(gauntletContextValue);

export function useGauntletContext() {
    return useContext(gauntletContext);
}

export async function getAssetData(path: string): Promise<ArrayBuffer> {
    return await asset_data(path);
}

export function getAssetDataSync(path: string): ArrayBuffer {
    return asset_data_blocking(path);
}

export function getPluginPreferences(): Record<string, any> {
    return gauntletContextValue.pluginPreferences()
}

export function getEntrypointPreferences(): Record<string, any> {
    return gauntletContextValue.entrypointPreferences()
}

export function showHudWindow(display: string): void {
    show_hud(display)
}

function createWidget(id: number | undefined, hostContext: HostContext, type: ComponentType, properties: Props, children: UiWidget[]): Instance {
    const props = Object.fromEntries(
        Object.entries(properties)
            .filter(([key, _]) => key !== "children")
    );

    const instance: Instance = {
        widgetId: id != undefined ? id : hostContext.nextId,
        widgetType: type,
        widgetProperties: props,
        widgetChildren: children,
        hostContext
    };

    if (id == undefined) {
        hostContext.nextId += 1
    }

    return instance
}

const componentModel = op_component_model();

export const createHostConfig = (): HostConfig<
    ComponentType,
    PropsWithChildren,
    RootUiWidget,
    Instance,
    TextInstance,
    SuspenseInstance,
    never,
    PublicInstance,
    HostContext,
    UpdatePayload,
    ChildSet,
    TimeoutHandle,
    NoTimeout
> => ({
    /*
     core items
    */
    createInstance: (
        type: ComponentType,
        props: PropsWithChildren,
        rootContainer: RootUiWidget,
        hostContext: HostContext,
        _internalHandle: OpaqueHandle,
    ): Instance => {
        op_log_trace("renderer_js_common", `createInstance is called, type: ${type}, props: ${Deno.inspect(props)}, rootContainer: ${Deno.inspect(rootContainer)}`)
        const instance = createWidget(undefined, hostContext, type, props, [])
        op_log_trace("renderer_js_common", `createInstance returned, widget: ${Deno.inspect(instance)}`)

        return instance;
    },

    createTextInstance: (
        text: string,
        rootContainer: RootUiWidget,
        hostContext: HostContext,
        _internalHandle: OpaqueHandle
    ): TextInstance => {
        op_log_trace("renderer_js_common", `createTextInstance is called, text: ${text}, rootContainer: ${Deno.inspect(rootContainer)}`)
        const textInstance = createWidget(undefined, hostContext, "gauntlet:text_part", { value: text }, [])
        op_log_trace("renderer_js_common", `createTextInstance returned, widget: ${Deno.inspect(textInstance)}`)

        return textInstance;
    },

    appendInitialChild: (parentInstance: Instance, child: Instance | TextInstance): void => {
        op_log_trace("renderer_js_common", `appendInitialChild is called, parentInstance: ${Deno.inspect(parentInstance)}, child: ${Deno.inspect(child)}`)

        parentInstance.widgetChildren.push(child)
    },

    finalizeInitialChildren: (
        instance: Instance,
        type: ComponentType,
        props: PropsWithChildren,
        _rootContainer: RootUiWidget,
        _hostContext: HostContext
    ): boolean => {
        op_log_trace("renderer_js_common", `finalizeInitialChildren is called, instance: ${Deno.inspect(instance)}, type: ${type}, props: ${Deno.inspect(props)}`)
        return false;
    },

    prepareUpdate: (
        instance: Instance,
        type: ComponentType,
        oldProps: PropsWithChildren,
        newProps: PropsWithChildren,
        _rootContainer: RootUiWidget,
        _hostContext: HostContext,
    ): UpdatePayload | null => {
        op_log_trace("renderer_js_common", `prepareUpdate is called, instance: ${Deno.inspect(instance)}, type: ${type}, oldProps: ${Deno.inspect(oldProps)}, newProps: ${Deno.inspect(newProps)}`)
        const diff = shallowDiff(oldProps, newProps);
        op_log_trace("renderer_js_common", `prepareUpdate shallowDiff returned: ${Deno.inspect(diff)}`)
        return diff;
    },
    shouldSetTextContent: (_type: ComponentType, _props: PropsWithChildren): boolean => {
        return false;
    },
    getRootHostContext: (_rootContainer: RootUiWidget): HostContext | null => {

        return new HostContext(1, componentModel);
    },
    getChildHostContext: (parentHostContext: HostContext, _type: ComponentType, _rootContainer: RootUiWidget): HostContext => {
        return parentHostContext;
    },
    getPublicInstance: (instance: Instance | TextInstance): PublicInstance => {
        return instance;
    },
    prepareForCommit: (_containerInfo: RootUiWidget): Record<string, any> | null => {
        return null;
    },
    resetAfterCommit: (_containerInfo: RootUiWidget): void => {
    },
    preparePortalMount: (_containerInfo: RootUiWidget): void => {
        throw new Error("React Portals are not supported")
    },
    scheduleTimeout(fn: (...args: unknown[]) => unknown, delay: number | undefined): TimeoutHandle {
        return setTimeout(fn, delay);
    },
    cancelTimeout(id: TimeoutHandle): void {
        clearTimeout(id)
    },
    noTimeout: -1,
    isPrimaryRenderer: true,
    getCurrentEventPriority: () => DefaultEventPriority,
    getInstanceFromNode(_node: any): ReactReconciler.Fiber | null | undefined {
        return undefined;
    },
    beforeActiveInstanceBlur: (): void => {
        throw Error("UNUSED")
    },
    afterActiveInstanceBlur: (): void => {
        throw Error("UNUSED")
    },
    prepareScopeUpdate: (_scopeInstance: any, _instance: any): void => {
        throw Error("UNUSED")
    },
    getInstanceFromScope: (_scopeInstance: any): null | Instance => {
        throw Error("UNUSED")
    },
    detachDeletedInstance: (_node: Instance): void => {
    },

    /*
     mutation items
    */
    supportsMutation: false,
    /*
     persistence items
    */
    supportsPersistence: true,

    cloneInstance(
        instance: Instance,
        updatePayload: UpdatePayload,
        type: ComponentType,
        oldProps: PropsWithChildren,
        newProps: PropsWithChildren,
        _internalInstanceHandle: OpaqueHandle,
        keepChildren: boolean,
        recyclableInstance: null | Instance,
    ): Instance {
        op_log_trace("renderer_js_persistence", `cloneInstance is called, instance: ${Deno.inspect(instance)}, updatePayload: ${Deno.inspect(updatePayload)}, type: ${type}, oldProps: ${Deno.inspect(oldProps)}, newProps: ${Deno.inspect(newProps)}, keepChildren: ${keepChildren}, recyclableInstance: ${Deno.inspect(recyclableInstance)}`)

        const recyclableId = recyclableInstance != null ? recyclableInstance.widgetId : undefined;

        let clonedInstance: Instance;

        if (keepChildren) {
            if (updatePayload !== null) {
                clonedInstance = createWidget(recyclableId, instance.hostContext, type, newProps, instance.widgetChildren)
            } else {
                clonedInstance = createWidget(recyclableId, instance.hostContext, type, oldProps, instance.widgetChildren)
            }
        } else {
            if (updatePayload !== null) {
                clonedInstance = createWidget(recyclableId, instance.hostContext, type, newProps, [])
            } else {
                clonedInstance = createWidget(recyclableId, instance.hostContext, type, oldProps, [])
            }
        }

        op_log_trace("renderer_js_persistence", `cloneInstance returned, widget: ${Deno.inspect(clonedInstance)}`)

        return clonedInstance;
    },

    createContainerChildSet(container: RootUiWidget): ChildSet {
        op_log_trace("renderer_js_persistence", `createContainerChildSet is called, container: ${Deno.inspect(container)}`)

        return []
    },

    appendChildToContainerChildSet(childSet: ChildSet, child: Instance | TextInstance): void {
        op_log_trace("renderer_js_persistence", `appendChildToContainerChildSet is called, childSet: ${Deno.inspect(childSet)}, child: ${Deno.inspect(child)}`)

        childSet.push(child);
    },

    finalizeContainerChildren(container: RootUiWidget, newChildren: ChildSet): void {
        op_log_trace("renderer_js_persistence", `finalizeContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren)}`)
    },

    replaceContainerChildren(container: RootUiWidget, newChildren: ChildSet): void {
        // op_log_info("renderer_js_persistence", `replaceContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren, { depth: Number.MAX_VALUE })}`)

        container.widgetChildren = newChildren

        const containerComponent = { content: newChildren.map(value => convertComponents(value)) }

        // op_log_info("renderer_js_persistence", `Converted container: ${Deno.inspect(containerComponent, { depth: Number.MAX_VALUE })}`)

        op_react_replace_view(
            gauntletContextValue.renderLocation(),
            gauntletContextValue.isBottommostView(),
            gauntletContextValue.entrypointId(),
            gauntletContextValue.entrypointName(),
            containerComponent
        )
    },

    cloneHiddenInstance(
        _instance: Instance,
        _type: ComponentType,
        _props: PropsWithChildren,
        _internalInstanceHandle: OpaqueHandle,
    ): Instance {
        throw new Error("NOT IMPLEMENTED")
    },

    cloneHiddenTextInstance(_instance: Instance, _text: ComponentType, _internalInstanceHandle: OpaqueHandle): TextInstance {
        throw new Error("NOT IMPLEMENTED")
    },

    /*
     hydration items
    */
    supportsHydration: false
});


function convertComponents(widget: UiWidget): any  {
    const component: any = {
        __id__: widget.widgetId,
        __type__: widget.widgetType,
        ...widget.widgetProperties
    }

    for (const [name, value] of Object.entries(component)) {
        if (typeof value === "function") {
            delete component[name]
        }
    }

    component.content = widget.widgetChildren
        .map(child => convertComponents(child))

    return component
}

function shallowDiff(oldObj: Record<string, any>, newObj: Record<string, any>): string[] | null {
    const uniqueProps = new Set([...Object.keys(oldObj), ...Object.keys(newObj)]);
    const diff = Array.from(uniqueProps)
        .filter(propName => propName != "children")
        .filter(propName => oldObj[propName] !== newObj[propName]);

    return diff.length === 0 ? null : diff;
}


const createTracedHostConfig = (hostConfig: any) => new Proxy(hostConfig, {
    get(target, propKey, _receiver) {
        const f = (target as any)[propKey];

        if (typeof f === 'undefined') {
            console.log('MethodTrace: Stubbing undefined property access for', propKey);

            return function _noop(...args: any[]) {
                console.log('MethodTrace Stub:', propKey, ...args.map(function (arg) {
                    return Deno.inspect(arg, {depth: 1});
                }));
            }
        }

        if (typeof f === 'function') {
            return function _traced(this: any, ...args: any[]) {
                console.log('MethodTrace:', propKey, ...args.map(function (arg) {
                    return Deno.inspect(arg, {depth: 1});
                }));

                return f.apply(this, args);
            }
        }

        return f;
    }
});

export function clearRenderer() {
    gauntletContextValue.clear()
}

export function render(entrypointId: string, entrypointName: string, renderLocation: RenderLocation, view: ReactNode): UiWidget {
    const hostConfig = createHostConfig();

    // const reconciler = ReactReconciler(createTracedHostConfig(hostConfig));
    const reconciler = ReactReconciler(hostConfig);

    const container: RootUiWidget = {
        widgetId: 0,
        widgetType: "gauntlet:root",
        widgetProperties: {},
        widgetChildren: [],
    };

    gauntletContextValue.reset(
        entrypointId,
        entrypointName,
        renderLocation,
        view,
        (node: ReactNode) => {
            reconciler.updateContainer(
                node,
                root,
                null,
                null
            );
        },
        () => {
            reconciler.updateContainer(
                null,
                root,
                null,
                null
            );
        }
    )

    const root = reconciler.createContainer(
        container,
        0,
        null,
        false,
        false,
        "",
        error => {
            console.error("Recoverable error occurred when rendering view", error)
        },
        null
    );

    gauntletContextValue.rerender(gauntletContextValue.topmostView())

    return container
}
