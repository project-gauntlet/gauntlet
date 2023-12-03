import ReactReconciler, {HostConfig, OpaqueHandle} from "react-reconciler";
import type React from 'react';
import {DefaultEventPriority} from 'react-reconciler/constants';

// @ts-expect-error does typescipt support such symbol declarations?
const denoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

type PublicInstance = Instance;
type HostContext = any;
type UpdatePayload = string[];
type TimeoutHandle = any;
type NoTimeout = -1;

// TODO add on not used methods: throw new Error("NOT IMPLEMENTED")

export const createHostConfig = (options: { mode: "mutation" | "persistent" }): HostConfig<
    ComponentType,
    Props,
    Container,
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
        props: Props,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle,
    ): Instance => {
        InternalApi.op_log_trace("renderer_js_common", `createInstance is called, type: ${type}, props: ${Deno.inspect(props)}, rootContainer: ${Deno.inspect(rootContainer)}`)
        const instance = InternalApi.op_react_create_instance(type, props);
        InternalApi.op_log_trace("renderer_js_common", `op_react_create_instance returned, widget: ${Deno.inspect(instance)}`)

        return instance;
    },

    createTextInstance: (
        text: string,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle
    ): TextInstance => {
        InternalApi.op_log_trace("renderer_js_common", `createTextInstance is called, text: ${text}, rootContainer: ${Deno.inspect(rootContainer)}`)
        const textInstance = InternalApi.op_react_create_text_instance(text);
        InternalApi.op_log_trace("renderer_js_common", `op_react_create_text_instance returned, widget: ${Deno.inspect(textInstance)}`)

        return textInstance;
    },

    appendInitialChild: (parentInstance: Instance, child: Instance | TextInstance): void => {
        InternalApi.op_log_trace("renderer_js_common", `appendInitialChild is called, parentInstance: ${Deno.inspect(parentInstance)}, child: ${Deno.inspect(child)}`)
        InternalApi.op_react_append_child(parentInstance, child)
    },

    finalizeInitialChildren: (
        instance: Instance,
        type: ComponentType,
        props: Props,
        rootContainer: Container,
        hostContext: HostContext
    ): boolean => {
        InternalApi.op_log_trace("renderer_js_common", `finalizeInitialChildren is called, instance: ${Deno.inspect(instance)}, type: ${type}, props: ${Deno.inspect(props)}`)
        return false;
    },

    prepareUpdate: (
        instance: Instance,
        type: ComponentType,
        oldProps: Props,
        newProps: Props,
        rootContainer: Container,
        hostContext: HostContext,
    ): UpdatePayload | null => {
        InternalApi.op_log_trace("renderer_js_common", `prepareUpdate is called, instance: ${Deno.inspect(instance)}, type: ${type}, oldProps: ${Deno.inspect(oldProps)}, newProps: ${Deno.inspect(newProps)}`)
        return shallowDiff(oldProps, newProps);
    },
    shouldSetTextContent: (type: ComponentType, props: Props): boolean => {
        return false;
    },
    getRootHostContext: (rootContainer: Container): HostContext | null => {
        return null;
    },
    getChildHostContext: (parentHostContext: HostContext, type: ComponentType, rootContainer: Container): HostContext => {
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
        throw new Error("React Portals are not supported")
    },
    scheduleTimeout(fn: (...args: unknown[]) => unknown, delay: number | undefined): TimeoutHandle {
        // TODO schedule timeout in tokio
        return undefined;
    },
    cancelTimeout(id: TimeoutHandle): void {
        // TODO cancel timeout in tokio
    },
    noTimeout: -1,
    isPrimaryRenderer: true,
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
    supportsMutation: isMutationMode(options.mode),

    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `appendChild is called, parentInstance: ${Deno.inspect(parentInstance)}, child: ${Deno.inspect(child)}`)

        InternalApi.op_react_append_child(parentInstance, child)
    },
    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `appendChildToContainer is called, container: ${Deno.inspect(container)}, child: ${Deno.inspect(child)}`)

        InternalApi.op_react_append_child(container, child)
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `insertBefore is called, parentInstance: ${Deno.inspect(parentInstance)}, child: ${Deno.inspect(child)}, beforeChild: ${Deno.inspect(beforeChild)}`)

        InternalApi.op_react_insert_before(parentInstance, child, beforeChild)
    },
    insertInContainerBefore(
        container: Container,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `insertInContainerBefore is called, container: ${Deno.inspect(container)}, child: ${Deno.inspect(child)}, beforeChild: ${Deno.inspect(beforeChild)}`)

        InternalApi.op_react_insert_before(container, child, beforeChild)
    },

    removeChild(
        parentInstance: Instance,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `removeChild is called, parentInstance: ${Deno.inspect(parentInstance)}, child: ${Deno.inspect(child)}`)

        InternalApi.op_react_remove_child(parentInstance, child)
    },
    removeChildFromContainer(
        container: Container,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `removeChildFromContainer is called, container: ${Deno.inspect(container)}, child: ${Deno.inspect(child)}`)

        InternalApi.op_react_remove_child(container, child)
    },


    commitUpdate(instance: Instance, updatePayload: UpdatePayload, type: ComponentType, prevProps: Props, nextProps: Props, internalHandle: ReactReconciler.OpaqueHandle): void {
        assertMutationMode(options.mode);

        InternalApi.op_log_trace("renderer_js_mutation", `commitUpdate is called, instance: ${Deno.inspect(instance)}, updatePayload: ${Deno.inspect(updatePayload)}, type: ${type}, prevProps: ${Deno.inspect(prevProps)}, nextProps: ${Deno.inspect(nextProps)}`)

        if (updatePayload.length) {
            const props = Object.fromEntries(
                updatePayload.map(propName => [propName, nextProps[propName]])
            );
            InternalApi.op_react_set_properties(instance, props);
        }
    },
    commitTextUpdate(textInstance: TextInstance, oldText: string, newText: string): void {
        assertMutationMode(options.mode);
        InternalApi.op_log_trace("renderer_js_mutation", `commitTextUpdate is called, textInstance: ${Deno.inspect(textInstance)}, oldText: ${oldText}, newText: ${newText}`)

        InternalApi.op_react_set_text(textInstance, newText)
    },

    hideInstance(instance: Instance): void {
        // TODO suspend support
        throw new Error("NOT IMPLEMENTED")
    },
    hideTextInstance(textInstance: TextInstance): void {
        // TODO suspend support
        throw new Error("NOT IMPLEMENTED")
    },
    unhideInstance(instance: Instance, props: Props): void {
        // TODO suspend support
        throw new Error("NOT IMPLEMENTED")
    },
    unhideTextInstance(textInstance: TextInstance, text: string): void {
        // TODO suspend support
        throw new Error("NOT IMPLEMENTED")
    },

    clearContainer: (container: Container): void => {
        InternalApi.op_log_trace("renderer_js_mutation", `clearContainer is called, container: ${Deno.inspect(container)}`)
    },

    /*
     persistence items
    */
    supportsPersistence: isPersistentMode(options.mode),

    cloneInstance(
        instance: Instance,
        updatePayload: UpdatePayload,
        type: ComponentType,
        oldProps: Props,
        newProps: Props,
        internalInstanceHandle: OpaqueHandle,
        keepChildren: boolean,
        recyclableInstance: null | Instance,
    ): Instance {
        assertPersistentMode(options.mode);

        InternalApi.op_log_trace("renderer_js_persistence", `cloneInstance is called, type: ${type}, newProps: ${Deno.inspect(newProps)}`)
        const cloned_instance = InternalApi.op_react_clone_instance(type, newProps);
        InternalApi.op_log_trace("renderer_js_persistence", `op_react_clone_instance returned, widget: ${Deno.inspect(cloned_instance)}`)

        return cloned_instance;
    },

    createContainerChildSet(container: Container): ChildSet {
        assertPersistentMode(options.mode);
        InternalApi.op_log_trace("renderer_js_persistence", `createContainerChildSet is called, container: ${Deno.inspect(container)}`)

        return []
    },

    appendChildToContainerChildSet(childSet: ChildSet, child: Instance | TextInstance): void {
        assertPersistentMode(options.mode);
        InternalApi.op_log_trace("renderer_js_persistence", `appendChildToContainerChildSet is called, childSet: ${Deno.inspect(childSet)}, child: ${Deno.inspect(child)}`)

        childSet.push(child);
    },

    finalizeContainerChildren(container: Container, newChildren: ChildSet): void {
        assertPersistentMode(options.mode);
        InternalApi.op_log_trace("renderer_js_persistence", `finalizeContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren)}`)
    },

    replaceContainerChildren(container: Container, newChildren: ChildSet): void {
        assertPersistentMode(options.mode);
        InternalApi.op_log_trace("renderer_js_persistence", `replaceContainerChildren is called, container: ${Deno.inspect(container)}, newChildren: ${Deno.inspect(newChildren)}`)
        InternalApi.op_react_replace_container_children(container, newChildren)
    },

    cloneHiddenInstance(
        instance: Instance,
        type: ComponentType,
        props: Props,
        internalInstanceHandle: OpaqueHandle,
    ): Instance {
        throw new Error("NOT IMPLEMENTED")
    },

    cloneHiddenTextInstance(instance: Instance, text: ComponentType, internalInstanceHandle: OpaqueHandle): TextInstance {
        throw new Error("NOT IMPLEMENTED")
    },

    /*
     hydration items
    */
    supportsHydration: false
});

const isPersistentMode = (mode: "mutation" | "persistent") => mode === "persistent";
const assertPersistentMode = (mode: "mutation" | "persistent") => {
    if (!isPersistentMode(mode)) {
        throw new Error("Wrong reconciler mode")
    }
}

const isMutationMode = (mode: "mutation" | "persistent") => mode === "mutation";
const assertMutationMode = (mode: "mutation" | "persistent") => {
    if (!isMutationMode(mode)) {
        throw new Error("Wrong reconciler mode")
    }
}

function shallowDiff(oldObj: Record<string, any>, newObj: Record<string, any>): string[] {
    const uniqueProps = new Set([...Object.keys(oldObj), ...Object.keys(newObj)]);
    return Array.from(uniqueProps)
        .filter(propName => oldObj[propName] !== newObj[propName]);
}


const createTracedHostConfig = (hostConfig: any) => new Proxy(hostConfig, {
    get(target, propKey, receiver) {
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

export function render(mode: "mutation" | "persistent", View: React.FC) {
    const hostConfig = createHostConfig({mode});

    // const reconciler = ReactReconciler(createTracedHostConfig(hostConfig));
    const reconciler = ReactReconciler(hostConfig);

    const root = reconciler.createContainer(
        InternalApi.op_react_get_container(),
        0,
        null,
        false,
        false,
        "custom",
        error => {
        },
        null
    );

    reconciler.updateContainer(
        <View/>,
        root,
        null,
        null
    );
}
