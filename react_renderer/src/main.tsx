import ReactReconciler, {HostConfig, OpaqueHandle} from "react-reconciler";
import React from 'react';
import {DefaultEventPriority} from 'react-reconciler/constants';
import Preview from "./preview";

type Type = keyof JSX.IntrinsicElements;
type Props = { children: any } & { [key: string]: any };

type SuspenseInstance = Instance;
type PublicInstance = Instance;
type HostContext = any;
type UpdatePayload = any;
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
const InternalApi: InternalApi = Deno[Deno.internal].core.ops;

type Container = Instance

declare interface Instance {
}

declare interface TextInstance {
}

declare interface InternalApi {
    op_gtk_get_container(): Container;
    op_gtk_create_instance(type: Type, props: Props): Instance;
    op_gtk_create_text_instance(text: string): TextInstance;
    op_gtk_append_child(parent: Instance, child: Instance | TextInstance): void;
    op_gtk_remove_child(parent: Instance, child: Instance | TextInstance): void;
    op_gtk_insert_before(
        parent: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void;
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
        console.log("createInstance")
        return InternalApi.op_gtk_create_instance(type, props);
    },

    createTextInstance: (
        text: string,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle
    ): TextInstance => {
        console.log("createTextInstance")
        return InternalApi.op_gtk_create_text_instance(text);
    },
    finalizeInitialChildren: (): boolean => {
        console.log("finalizeInitialChildren")
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
        console.log("prepareUpdate")
        // TODO diffing of props is done here
        return null;
    },
    shouldSetTextContent: (type: Type, props: Props): boolean => {
        return false; // in gtk arbitrary node cannot contain text, only Label can
    },
    getRootHostContext: (rootContainer: Container): HostContext | null => {
        console.log("getRootHostContext")
        return null;
    },
    getChildHostContext: (parentHostContext: HostContext, type: Type, rootContainer: Container): HostContext => {
        console.log("getChildHostContext")
        return parentHostContext;
    },
    getPublicInstance: (instance: Instance | TextInstance): PublicInstance => {
        console.log("getPublicInstance")
        return instance;
    },
    prepareForCommit: (containerInfo: Container): Record<string, any> | null => {
        console.log("prepareForCommit")
        return null;
    },
    resetAfterCommit: (containerInfo: Container): void => {
        console.log("resetAfterCommit")
    },
    preparePortalMount: (containerInfo: Container): void => {
        console.log("preparePortalMount")
    },
    scheduleTimeout(fn: (...args: unknown[]) => unknown, delay: number | undefined): TimeoutHandle {
        // TODO schedule timeout in tokio
        console.log("scheduleTimeout")
        return undefined;
    },
    cancelTimeout(id: TimeoutHandle): void {
        // TODO cancel timeout in tokio
        console.log("cancelTimeout")
    },
    noTimeout: -1,
    isPrimaryRenderer: true, // we have single separate renderer per view
    getCurrentEventPriority: () => DefaultEventPriority,
    getInstanceFromNode(node: any): ReactReconciler.Fiber | null | undefined {
        console.log("getInstanceFromNode")
        return undefined;
    },
    beforeActiveInstanceBlur: (): void => {
        console.log("beforeActiveInstanceBlur")
    },
    afterActiveInstanceBlur: (): void => {
        console.log("afterActiveInstanceBlur")
    },
    prepareScopeUpdate: (scopeInstance: any, instance: any): void => {
        console.log("prepareScopeUpdate")
    },
    getInstanceFromScope: (scopeInstance: any): null | Instance => {
        console.log("getInstanceFromScope")
        return null;
    },
    detachDeletedInstance: (node: Instance): void => {
        console.log("detachDeletedInstance")
    },

    /*
     mutation items
    */
    supportsMutation: true,
    appendInitialChild: (parentInstance: Instance, child: Instance | TextInstance): void => {
        console.log("appendInitialChild", parentInstance, child)
        InternalApi.op_gtk_append_child(parentInstance, child)
    },
    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        console.log("appendChild", parentInstance, child)
        InternalApi.op_gtk_append_child(parentInstance, child)
    },
    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        console.log("appendChildToContainer", container, child)
        InternalApi.op_gtk_append_child(container, child)
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        InternalApi.op_gtk_insert_before(parentInstance, child, beforeChild)
    },
    insertInContainerBefore(
        container: Container,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        InternalApi.op_gtk_insert_before(container, child, beforeChild)
    },

    removeChild(
        parentInstance: Instance,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        InternalApi.op_gtk_remove_child(parentInstance, child)
    },
    removeChildFromContainer(
        container: Container,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        InternalApi.op_gtk_remove_child(container, child)
    },


    commitUpdate(instance: Instance, updatePayload: UpdatePayload, type: Type, prevProps: Props, nextProps: Props, internalHandle: ReactReconciler.OpaqueHandle): void {
        console.log("commitUpdate")
    },
    commitTextUpdate(textInstance: TextInstance, oldText: string, newText: string): void {
        console.log("commitTextUpdate")
    },

    hideInstance(instance: Instance): void {
        // TODO suspend support
        console.log("hideInstance")
    },
    hideTextInstance(textInstance: TextInstance): void {
        // TODO suspend support
        console.log("hideTextInstance")
    },
    unhideInstance(instance: Instance, props: Props): void {
        // TODO suspend support
        console.log("unhideInstance")
    },
    unhideTextInstance(textInstance: TextInstance, text: string): void {
        // TODO suspend support
        console.log("unhideTextInstance")
    },

    clearContainer: (container: Container): void => {
        console.log("clearContainer")
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

const reconciler = ReactReconciler(hostConfig);

const root = reconciler.createContainer(InternalApi.op_gtk_get_container(), 0, null, false, false, "custom", error => {
}, null);

// console.dir(root)
reconciler.updateContainer(<Preview/>, root, null, null);

setTimeout(() => {
    console.log("test timeout console")
}, 3000)