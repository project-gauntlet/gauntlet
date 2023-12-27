import ReactReconciler, {HostConfig, OpaqueHandle} from "react-reconciler";
import type React from 'react';
import {DefaultEventPriority} from 'react-reconciler/constants';

// @ts-expect-error does typescript support such symbol declarations?
const denoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

type PublicInstance = Instance;
type HostContext = RootContext;
type TimeoutHandle = number;
type NoTimeout = -1;


function createWidget(hostContext: HostContext, type: ComponentType, properties: Props = {}, children: UiWidget[] = []): Instance {
    const props = Object.fromEntries(
        Object.entries(properties)
            .filter(([key, _]) => key !== "children")
    );

    const nextId = hostContext.nextId;
    const instance: Instance = {
        widgetId: nextId,
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
        return { nextId: 1 };
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

        // TODO validate
        // TODO     validate_properties(&state, &instance.widget_type, &new_props)?;

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
        InternalApi.op_log_trace("renderer_js_persistence", `replaceContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren)}`)
        container.widgetChildren = newChildren
        InternalApi.op_react_replace_container_children(container, newChildren)
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

export function render(frontend: string, View: React.FC): RootUiWidget {
    // specific frontend are implemented separately, it seems it is not feasible to do generic implementation
    if (frontend !== "default") {
        throw new Error("NOT SUPPORTED")
    }

    const hostConfig = createHostConfig();

    // const reconciler = ReactReconciler(createTracedHostConfig(hostConfig));
    const reconciler = ReactReconciler(hostConfig);

    const container: RootUiWidget = {
        widgetId: 0,
        widgetType: "gauntlet:root",
        widgetProperties: {},
        widgetChildren: [],
    };

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

    reconciler.updateContainer(
        <View/>,
        root,
        null,
        null
    );

    return container
}
