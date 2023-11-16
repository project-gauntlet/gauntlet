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
    ChildSet,
    TimeoutHandle,
    NoTimeout
> => ({
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
        return InternalApi.op_react_create_instance(type, props);
    },

    createTextInstance: (
        text: string,
        rootContainer: Container,
        hostContext: HostContext,
        internalHandle: OpaqueHandle
    ): TextInstance => {
        return InternalApi.op_react_create_text_instance(text);
    },

    appendInitialChild: (parentInstance: Instance, child: Instance | TextInstance): void => {
        InternalApi.op_react_append_child(parentInstance, child)
    },

    finalizeInitialChildren: (
        instance: Instance,
        type: Type,
        props: Props,
        rootContainer: Container,
        hostContext: HostContext
    ): boolean => {
        // instance.then(value => InternalApi.op_react_set_properties(value, props));
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
        return false;
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
    supportsMutation: isMutationMode(options.mode),

    appendChild(parentInstance: Instance, child: Instance | TextInstance): void {
        assertMutationMode(options.mode);

        InternalApi.op_react_append_child(parentInstance, child)
    },
    appendChildToContainer(container: Container, child: Instance | TextInstance): void {
        assertMutationMode(options.mode);

        InternalApi.op_react_append_child(container, child)
    },

    insertBefore(
        parentInstance: Instance,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);

        InternalApi.op_react_insert_before(parentInstance, child, beforeChild)
    },
    insertInContainerBefore(
        container: Container,
        child: Instance | TextInstance,
        beforeChild: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);

        InternalApi.op_react_insert_before(container, child, beforeChild)
    },

    removeChild(
        parentInstance: Instance,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);

        InternalApi.op_react_remove_child(parentInstance, child)
    },
    removeChildFromContainer(
        container: Container,
        child: Instance | TextInstance | SuspenseInstance
    ): void {
        assertMutationMode(options.mode);

        InternalApi.op_react_remove_child(container, child)
    },


    commitUpdate(instance: Instance, updatePayload: UpdatePayload, type: Type, prevProps: Props, nextProps: Props, internalHandle: ReactReconciler.OpaqueHandle): void {
        assertMutationMode(options.mode);

        if (updatePayload.length) {
            const props = Object.fromEntries(
                updatePayload.map(propName => [propName, nextProps[propName]])
            );
            InternalApi.op_react_set_properties(instance, props);
        }
    },
    commitTextUpdate(textInstance: TextInstance, oldText: string, newText: string): void {
        assertMutationMode(options.mode);

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
    },

    /*
     persistence items
    */
    supportsPersistence: isPersistentMode(options.mode),

    cloneInstance(
        instance: Instance,
        updatePayload: UpdatePayload,
        type: Type,
        oldProps: Props,
        newProps: Props,
        internalInstanceHandle: OpaqueHandle,
        keepChildren: boolean,
        recyclableInstance: null | Instance,
    ): Instance {
        assertPersistentMode(options.mode);

        return InternalApi.op_react_clone_instance(type, newProps);
    },

    createContainerChildSet(container: Container): ChildSet {
        assertPersistentMode(options.mode);

        return []
    },

    appendChildToContainerChildSet(childSet: ChildSet, child: Instance | TextInstance): void {
        assertPersistentMode(options.mode);

        childSet.push(child);
    },

    finalizeContainerChildren(container: Container, newChildren: ChildSet): void {
        assertPersistentMode(options.mode);
    },

    replaceContainerChildren(container: Container, newChildren: ChildSet): void {
        assertPersistentMode(options.mode);

        InternalApi.op_react_replace_container_children(container, newChildren)
    },

    cloneHiddenInstance(
        instance: Instance,
        type: Type,
        props: Props,
        internalInstanceHandle: OpaqueHandle,
    ): Instance {
        throw new Error("NOT IMPLEMENTED")
    },

    cloneHiddenTextInstance(instance: Instance, text: Type, internalInstanceHandle: OpaqueHandle): TextInstance {
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
