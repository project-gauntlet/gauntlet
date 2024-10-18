import ReactReconciler, { HostConfig, OpaqueHandle } from "react-reconciler";
import { createContext, FC, ReactNode, useContext } from 'react';
import { DefaultEventPriority } from 'react-reconciler/constants';

// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

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
    private _clear: (() => void) | undefined;

    reset(entrypointId: string, renderLocation: RenderLocation, view: ReactNode, rerender: (node: ReactNode) => void, clear: () => void) {
        this._entrypointId = entrypointId
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
        return InternalApi.get_entrypoint_preferences(this.entrypointId())
    }

    pluginPreferences = () => {
        return InternalApi.get_plugin_preferences()
    }
}

const gauntletContextValue = new GauntletContextValue()
const gauntletContext = createContext(gauntletContextValue);

export function useGauntletContext() {
    return useContext(gauntletContext);
}

export async function getAssetData(path: string): Promise<ArrayBuffer> {
    const vecU8 = await InternalApi.asset_data(path);
    return new Uint8Array(vecU8).buffer; // FIXME move array creation into rust if possible
}

export function getAssetDataSync(path: string): ArrayBuffer {
    const vecU8 = InternalApi.asset_data_blocking(path);
    return new Uint8Array(vecU8).buffer;
}

export function getPluginPreferences(): Record<string, any> {
    return gauntletContextValue.pluginPreferences()
}

export function getEntrypointPreferences(): Record<string, any> {
    return gauntletContextValue.entrypointPreferences()
}

export function showHudWindow(display: string): void {
    InternalApi.show_hud(display)
}

function createWidget(hostContext: HostContext, type: ComponentType, properties: Props, children: UiWidget[] = []): Instance {
    const props = Object.fromEntries(
        Object.entries(properties)
            .filter(([key, _]) => key !== "children")
    );

    const instance: Instance = {
        widgetId: hostContext.nextId,
        widgetType: type,
        widgetProperties: props,
        widgetChildren: children,
        hostContext
    };
    hostContext.nextId += 1
    return instance
}

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
        InternalApi.op_log_trace("renderer_js_common", `createInstance is called, type: ${type}, props: ${Deno.inspect(props)}, rootContainer: ${Deno.inspect(rootContainer)}`)
        const instance = createWidget(hostContext, type, props)
        InternalApi.op_log_trace("renderer_js_common", `createInstance returned, widget: ${Deno.inspect(instance)}`)

        return instance;
    },

    createTextInstance: (
        text: string,
        rootContainer: RootUiWidget,
        hostContext: HostContext,
        _internalHandle: OpaqueHandle
    ): TextInstance => {
        InternalApi.op_log_trace("renderer_js_common", `createTextInstance is called, text: ${text}, rootContainer: ${Deno.inspect(rootContainer)}`)
        const textInstance = createWidget(hostContext, "gauntlet:text_part", { value: text })
        InternalApi.op_log_trace("renderer_js_common", `createTextInstance returned, widget: ${Deno.inspect(textInstance)}`)

        return textInstance;
    },

    appendInitialChild: (parentInstance: Instance, child: Instance | TextInstance): void => {
        InternalApi.op_log_trace("renderer_js_common", `appendInitialChild is called, parentInstance: ${Deno.inspect(parentInstance)}, child: ${Deno.inspect(child)}`)

        parentInstance.widgetChildren.push(child)
    },

    finalizeInitialChildren: (
        instance: Instance,
        type: ComponentType,
        props: PropsWithChildren,
        _rootContainer: RootUiWidget,
        _hostContext: HostContext
    ): boolean => {
        InternalApi.op_log_trace("renderer_js_common", `finalizeInitialChildren is called, instance: ${Deno.inspect(instance)}, type: ${type}, props: ${Deno.inspect(props)}`)
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
        InternalApi.op_log_trace("renderer_js_common", `prepareUpdate is called, instance: ${Deno.inspect(instance)}, type: ${type}, oldProps: ${Deno.inspect(oldProps)}, newProps: ${Deno.inspect(newProps)}`)
        const diff = shallowDiff(oldProps, newProps);
        InternalApi.op_log_trace("renderer_js_common", `prepareUpdate shallowDiff returned: ${Deno.inspect(diff)}`)
        return diff;
    },
    shouldSetTextContent: (_type: ComponentType, _props: PropsWithChildren): boolean => {
        return false;
    },
    getRootHostContext: (_rootContainer: RootUiWidget): HostContext | null => {
        const componentModel = InternalApi.op_component_model();

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
        InternalApi.op_log_trace("renderer_js_persistence", `cloneInstance is called, instance: ${Deno.inspect(instance)}, updatePayload: ${Deno.inspect(updatePayload)}, type: ${type}, oldProps: ${Deno.inspect(oldProps)}, newProps: ${Deno.inspect(newProps)}, keepChildren: ${keepChildren}, recyclableInstance: ${Deno.inspect(recyclableInstance)}`)

        let clonedInstance: Instance;

        if (keepChildren) {
            if (updatePayload !== null) {
                clonedInstance = createWidget(instance.hostContext, type, newProps, instance.widgetChildren)
            } else {
                clonedInstance = createWidget(instance.hostContext, type, oldProps, instance.widgetChildren)
            }
        } else {
            if (updatePayload !== null) {
                clonedInstance = createWidget(instance.hostContext, type, newProps, [])
            } else {
                clonedInstance = createWidget(instance.hostContext, type, oldProps, [])
            }
        }

        InternalApi.op_log_trace("renderer_js_persistence", `cloneInstance returned, widget: ${Deno.inspect(clonedInstance)}`)

        return clonedInstance;
    },

    createContainerChildSet(container: RootUiWidget): ChildSet {
        InternalApi.op_log_trace("renderer_js_persistence", `createContainerChildSet is called, container: ${Deno.inspect(container)}`)

        return []
    },

    appendChildToContainerChildSet(childSet: ChildSet, child: Instance | TextInstance): void {
        InternalApi.op_log_trace("renderer_js_persistence", `appendChildToContainerChildSet is called, childSet: ${Deno.inspect(childSet)}, child: ${Deno.inspect(child)}`)

        childSet.push(child);
    },

    finalizeContainerChildren(container: RootUiWidget, newChildren: ChildSet): void {
        InternalApi.op_log_trace("renderer_js_persistence", `finalizeContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren)}`)
    },

    replaceContainerChildren(container: RootUiWidget, newChildren: ChildSet): void {
        // TODO Deno.inspect is always executed
        InternalApi.op_log_trace("renderer_js_persistence", `replaceContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren, { depth: Number.MAX_VALUE })}`)

        container.widgetChildren = newChildren

        InternalApi.op_react_replace_view(gauntletContextValue.renderLocation(), gauntletContextValue.isBottommostView(), gauntletContextValue.entrypointId(), container)
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

export function render(entrypointId: string, renderLocation: RenderLocation, view: ReactNode): UiWidget {
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
