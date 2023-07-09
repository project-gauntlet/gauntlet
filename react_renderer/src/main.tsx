import ReactReconciler, {HostConfig, OpaqueHandle} from "react-reconciler";
import React from 'react';
import {DefaultEventPriority} from 'react-reconciler/constants';
import Preview from "./preview";

type Type = keyof JSX.IntrinsicElements;
type Props = { children: any } & { [key: string]: any };

type SuspenseInstance = never;
type PublicInstance = Instance;
type HostContext = any;
type UpdatePayload = string[];
type TimeoutHandle = any;
type NoTimeout = -1;

declare global {
    namespace JSX {
        interface IntrinsicElements {
            box: {}
            button1: { onClick?: () => void, children: string }
            // TODO remove default html IntrinsicElements
        }
    }
}


// @ts-expect-error "Deno[Deno.internal]" is not a public interface
const denoCore = Deno[Deno.internal].core;
// @ts-expect-error // TODO "Deno" does not have typings
const denoInspect = Deno.inspect;

const InternalApi: InternalApi = denoCore.ops;

type Container = Instance
type Instance = Promise<GuiWidget>
type TextInstance = Promise<GuiWidget>
type InstanceSync = GuiWidget
type TextInstanceSync = GuiWidget

declare interface GuiWidget {
}

declare interface InternalApi {
    op_gtk_get_container(): Container;
    op_gtk_create_instance(type: Type): Instance;
    op_gtk_create_text_instance(text: string): TextInstance;

    op_gtk_append_child(parent: InstanceSync, child: InstanceSync | TextInstanceSync): void;
    op_gtk_remove_child(parent: InstanceSync, child: InstanceSync | TextInstanceSync): void;
    op_gtk_insert_before(
        parent: InstanceSync,
        child: InstanceSync | TextInstanceSync | SuspenseInstance,
        beforeChild: InstanceSync | TextInstanceSync | SuspenseInstance
    ): void;

    op_gtk_set_properties(instance: InstanceSync, child: Record<string, any>): void;
    op_gtk_set_text(instance: InstanceSync, text: string): void;
    op_call_event_listener(instance: InstanceSync, eventName: string): void;
}

const hostConfig: HostConfig<
    Type,
    Props,
    Container,
    Instance,
    TextInstance,
    SuspenseInstance,
    never,
    PublicInstance,
    HostContext,
    UpdatePayload,
    never,
    TimeoutHandle,
    NoTimeout
> = {
    /*
     core items
    */
    createInstance: (
        type: Type,
        props: Props,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle,
    ): Instance => {
        return InternalApi.op_gtk_create_instance(type);
    },

    createTextInstance: (
        text: string,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle
    ): TextInstance => {
        return InternalApi.op_gtk_create_text_instance(text);
    },
    finalizeInitialChildren: (
        instance: Instance,
        type: Type,
        props: Props,
        rootContainer: Container,
        hostContext: HostContext
    ): boolean => {
        instance.then(value => InternalApi.op_gtk_set_properties(value, props));
        return false;
    },
    prepareUpdate: (
        instance: Instance,
        type: Type,
        oldProps: Props,
        newProps: Props,
        rootContainer: Container,
        hostContext: HostContext,
    ): UpdatePayload | null => {
        return shallowDiff(oldProps, newProps);
    },
    shouldSetTextContent: (type: Type, props: Props): boolean => {
        return false; // in gtk arbitrary node cannot contain text, only Label can
    },
    getRootHostContext: (rootContainer: Container): HostContext | null => {
        return null;
    },
    getChildHostContext: (parentHostContext: HostContext, type: Type, rootContainer: Container): HostContext => {
        return parentHostContext;
    },
    getPublicInstance: (instance: Instance | TextInstance): PublicInstance => {
        return instance;
    },
    prepareForCommit: (containerInfo: Container): Record<string, any> | null => {
        return null;
    },
    resetAfterCommit: (containerInfo: Container): void => {
    },
    preparePortalMount: (containerInfo: Container): void => {
    },
    scheduleTimeout(fn: (...args: unknown[]) => unknown, delay: number | undefined): TimeoutHandle {
        // TODO schedule timeout in tokio
        return undefined;
    },
    cancelTimeout(id: TimeoutHandle): void {
        // TODO cancel timeout in tokio
    },
    noTimeout: -1,
    isPrimaryRenderer: true, // we have single separate renderer per view
    getCurrentEventPriority: () => DefaultEventPriority,
    getInstanceFromNode(node: any): ReactReconciler.Fiber | null | undefined {
        return undefined;
    },
    beforeActiveInstanceBlur: (): void => {
    },
    afterActiveInstanceBlur: (): void => {
    },
    prepareScopeUpdate: (scopeInstance: any, instance: any): void => {
    },
    getInstanceFromScope: (scopeInstance: any): null | Instance => {
        return null;
    },
    detachDeletedInstance: (node: Instance): void => {
    },

    /*
     mutation items
    */
    supportsMutation: true,
    appendInitialChild: (parentInstance: Instance, child: Instance | TextInstance): void => {
        Promise.all([parentInstance, child])
            .then(([resolvedParent, resolvedChild]) => {
                InternalApi.op_gtk_append_child(resolvedParent, resolvedChild)
            })
    },
    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        Promise.all([parentInstance, child])
            .then(([resolvedParent, resolvedChild]) => {
                InternalApi.op_gtk_append_child(resolvedParent, resolvedChild)
            })
    },
    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        Promise.all([container, child])
            .then(([resolvedContainer, resolvedChild]) => {
                InternalApi.op_gtk_append_child(resolvedContainer, resolvedChild)
            })
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        Promise.all([parentInstance, child, beforeChild])
            .then(([resolvedParentInstance, resolvedChild, resolvedBeforeChild]) => {
                InternalApi.op_gtk_insert_before(resolvedParentInstance, resolvedChild, resolvedBeforeChild)
            })
    },
    insertInContainerBefore(
        container: Container,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        Promise.all([container, child, beforeChild])
            .then(([resolvedContainer, resolvedChild, resolvedBeforeChild]) => {
                InternalApi.op_gtk_insert_before(resolvedContainer, resolvedChild, resolvedBeforeChild)
            })
    },

    removeChild(
        parentInstance: Instance,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        Promise.all([parentInstance, child])
            .then(([resolvedParent, resolvedChild]) => {
                InternalApi.op_gtk_remove_child(resolvedParent, resolvedChild)
            })
    },
    removeChildFromContainer(
        container: Container,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        Promise.all([container, child])
            .then(([resolvedContainer, resolvedChild]) => {
                InternalApi.op_gtk_remove_child(resolvedContainer, resolvedChild)
            })
    },


    commitUpdate(instance: Instance, updatePayload: UpdatePayload, type: Type, prevProps: Props, nextProps: Props, internalHandle: ReactReconciler.OpaqueHandle): void {
        if (updatePayload.length) {
            const props = Object.fromEntries(
                updatePayload.map(propName => [propName, nextProps[propName]])
            );
            instance.then(value => InternalApi.op_gtk_set_properties(value, props));
        }
    },
    commitTextUpdate(textInstance: TextInstance, oldText: string, newText: string): void {
        textInstance.then(value => InternalApi.op_gtk_set_text(value, newText))
    },

    hideInstance(instance: Instance): void {
        // TODO suspend support
    },
    hideTextInstance(textInstance: TextInstance): void {
        // TODO suspend support
    },
    unhideInstance(instance: Instance, props: Props): void {
        // TODO suspend support
    },
    unhideTextInstance(textInstance: TextInstance, text: string): void {
        // TODO suspend support
    },

    clearContainer: (container: Container): void => {
    },

    /*
     persistence items
    */
    supportsPersistence: false,

    /*
     hydration items
    */
    supportsHydration: false
};

function shallowDiff(oldObj: Record<string, any>, newObj: Record<string, any>): string[] {
    const uniqueProps = new Set([...Object.keys(oldObj), ...Object.keys(newObj)]);
    return Array.from(uniqueProps)
        .filter(propName => oldObj[propName] !== newObj[propName]);
}


const tracedHostConfig = new Proxy(hostConfig, {
    get(target, propKey, receiver) {
        const f = (target as any)[propKey];

        if (typeof f === 'undefined') {
            console.log('MethodTrace: Stubbing undefined property access for', propKey);

            return function _noop(...args: any[]) {
                console.log('MethodTrace Stub:', propKey, ...args.map(function (arg) {
                    return denoInspect(arg, {depth: 1});
                }));
            }
        }

        if (typeof f === 'function') {
            return function _traced(this: any, ...args: any[]) {
                console.log('MethodTrace:', propKey, ...args.map(function(arg) {
                    return denoInspect(arg, {depth: 1});
                }));

                return f.apply(this, args);
            }
        }

        return f;
    }
});

const reconciler = ReactReconciler(tracedHostConfig);

const root = reconciler.createContainer(InternalApi.op_gtk_get_container(), 0, null, false, false, "custom", error => {
}, null);

reconciler.updateContainer(<Preview/>, root, null, null);

(async () => {
    // noinspection InfiniteLoopJS
    while (true) {
        const guiEvent = await denoCore.opAsync("op_get_next_pending_gui_event");
        InternalApi.op_call_event_listener(guiEvent.widget, guiEvent.event_name)
    }
})();