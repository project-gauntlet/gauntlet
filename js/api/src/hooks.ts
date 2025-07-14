import { ReactNode, useRef, useId, useState, useCallback, useEffect, MutableRefObject, Dispatch, SetStateAction } from 'react';
// @ts-ignore TODO how to add declaration for this?
import { useGauntletContext } from "ext:gauntlet/renderer.js";

export function useNavigation(): { popView: () => void, pushView: (component: ReactNode) => void } {
    const { popView, pushView }: { popView: () => void, pushView: (component: ReactNode) => void } = useGauntletContext();

    return {
        popView: () => {
            popView()
        },
        pushView: (component: ReactNode) => {
            pushView(component)
        }
    }
}

export function usePluginPreferences<T extends Record<string, any>>(): T {
    const { pluginPreferences }: { pluginPreferences: () => T } = useGauntletContext();

    return pluginPreferences()
}

export function useEntrypointPreferences<T extends Record<string, any>>(): T {
    const { entrypointPreferences }: { entrypointPreferences: () => T } = useGauntletContext();

    return entrypointPreferences()
}

export type AsyncState<T> = {
    isLoading: boolean;
    error?: unknown;
    data?: T;
};

export type MutatePromiseFn<T, R> = (
    asyncUpdate: Promise<R>,
    options?: {
        optimisticUpdate?: (data: T | undefined) => T; // undefined, if options.execute is false and function was never called, needs to be pure
        rollbackOnError?: boolean | ((data: T | undefined) => T); // only used if optimisticUpdate is specified, needs to be pure
        shouldRevalidateAfter?: boolean; // only matters for successful updates
    },
) => Promise<R>;

export function usePromise<Return, Args extends unknown[], R = unknown>(
    fn: (...args: Args) => Promise<Return>,
    args?: Args,
    options?: {
        abortable?: MutableRefObject<AbortController | undefined>;
        execute?: boolean;
        onError?: (error: unknown) => void;
        onData?: (data: Return) => void;
        onWillExecute?: (...args: Args) => void;
    },
): AsyncState<Return> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<Return, R>;
} {
    const execute = options?.execute !== false; // execute by default

    const [state, setState] = useState<AsyncState<Return>>({ isLoading: execute });

    return usePromiseInternal(
        fn,
        state,
        setState,
        args || ([] as any),
        execute,
        options?.abortable,
        options?.onError,
        options?.onData,
        options?.onWillExecute
    )
}

export function useCachedPromise<Return, Args extends unknown[], R = unknown>(
    fn: (...args: Args) => Promise<Return>,
    args?: Args,
    options?: {
        initialState?: Return | (() => Return),
        abortable?: MutableRefObject<AbortController | undefined>;
        execute?: boolean;
        onError?: (error: unknown) => void;
        onData?: (data: Return) => void;
        onWillExecute?: (...args: Args) => void;
    },
): AsyncState<Return> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<Return, R>;
} {
    const execute = options?.execute !== false; // execute by default

    const id = useId();

    const { entrypointId }: { entrypointId: () => string } = useGauntletContext();

    // same store is fetched and updated between command runs
    const [state, setState] = useCache<AsyncState<Return>>("useCachedPromise" + entrypointId() + id, (): AsyncState<Return> => {
        const initialState = options?.initialState;
        if (initialState) {
            if (initialState instanceof Function) {
                return { isLoading: execute, data: initialState() }
            } else {
                return { isLoading: execute, data: initialState }
            }
        } else {
             return { isLoading: execute }
        }
    });

    return usePromiseInternal(
        fn,
        state,
        setState,
        args || ([] as any),
        execute,
        options?.abortable,
        options?.onError,
        options?.onData,
        options?.onWillExecute
    )
}

function usePromiseInternal<Return, Args extends unknown[], R = unknown>(
    fn: (...args: Args) => Promise<Return>,
    state: AsyncState<Return>,
    setState: Dispatch<SetStateAction<AsyncState<Return>>>,
    args: Args,
    execute: boolean,
    abortable?: MutableRefObject<AbortController | undefined>,
    onError?: (error: unknown) => void,
    onData?: (data: Return) => void,
    onWillExecute?: (...args: Args) => void,
): AsyncState<Return> & {
    revalidate: () => void; // will execute even if options.execute is false
    mutate: MutatePromiseFn<Return, R>; // will execute even if options.execute is false
} {

    const promiseRef = useRef<Promise<any>>();

    useEffect(() => {
        return () => {
            abortable?.current?.abort();
        };
    }, [abortable]);

    const callback = useCallback(async (...args: Args): Promise<void> => {
        if (abortable) {
            abortable.current?.abort();
            abortable.current = new AbortController()
        }

        onWillExecute?.(...args);

        const promise = fn(...args);

        setState(prevState => ({ ...prevState, isLoading: true }));

        promiseRef.current = promise;

        let promiseResult: Return;
        try {
            promiseResult = await promise;
        } catch (error) {
            // We dont want to handle result/error of non-latest function
            // this approach helps to avoid race conditions
            if (promise === promiseRef.current) {
                setState({ error, isLoading: false })

                if (abortable) {
                    abortable.current = undefined;
                }

                console.error("Error happened when executing promise: ", error)

                onError?.(error);
            }
            return
        }

        // We dont want to handle result/error of non-latest function
        // this approach helps to avoid race conditions
        if (promise === promiseRef.current) {
            setState({ data: promiseResult, isLoading: false });

            if (abortable) {
                abortable.current = undefined;
            }

            onData?.(promiseResult)
        }
    }, args);

    useEffect(() => {
        if (execute) {
            callback(...args);
        }
    }, [callback, execute]);

    return {
        revalidate: () => {
            callback(...args);
        },
        mutate: async (
            asyncUpdate: Promise<R>,
            options?: {
                optimisticUpdate?: (data: Return | undefined) => Return;
                rollbackOnError?: boolean | ((data: Return | undefined) => Return);
                shouldRevalidateAfter?: boolean;
            },
        ): Promise<R> => {
            const prevData = state.data;

            const optimisticUpdate = options?.optimisticUpdate;
            const rollbackOnError = options?.rollbackOnError;
            const shouldRevalidateAfter = options?.shouldRevalidateAfter !== false;

            if (optimisticUpdate) {
                const newData = optimisticUpdate(state.data);
                setState({ data: newData, isLoading: true })

                try {
                    const asyncUpdateResult = await asyncUpdate;

                    if (shouldRevalidateAfter) {
                        callback(...args);
                    } else {
                        // set loading false, only when not revalidating, because revalidate will unset it itself
                        setState(prevState => ({ ...prevState, isLoading: false }));
                    }

                    return asyncUpdateResult
                } catch (e) {
                    switch (typeof rollbackOnError) {
                        case "undefined": {
                            setState({ data: prevData, isLoading: false })
                            break;
                        }
                        case "boolean": {
                            if (rollbackOnError) {
                                setState({ data: prevData, isLoading: false })
                            }
                            break;
                        }
                        case "function": {
                            const rolledBackData = rollbackOnError(state.data);
                            setState({ data: rolledBackData, isLoading: false })
                            break;
                        }
                    }

                    throw e
                }
            } else {
                setState(prevState => ({ ...prevState, isLoading: true }));

                const asyncUpdateResult = await asyncUpdate;

                if (shouldRevalidateAfter) {
                    callback(...args);
                } else {
                    // set loading false, only when not revalidating, because revalidate will unset it itself
                    setState(prevState => ({ ...prevState, isLoading: false }));
                }

                return asyncUpdateResult
            }
        },
        ...state
    };
}

// persistent, uses localStorage under the hood
export function useStorage<T>(key: string, initialState: T | (() => T)): [T, Dispatch<SetStateAction<T>>] {
    return useWebStorage(key, initialState, localStorage)
}

// ephemeral, uses sessionStorage under the hood
export function useCache<T>(key: string, initialState: T | (() => T)): [T, Dispatch<SetStateAction<T>>] {
    return useWebStorage(key, initialState, sessionStorage)
}

// keys are shared per plugin, across all entrypoints
// uses JSON.serialize
function useWebStorage<T>(
    key: string,
    initialState: T | (() => T),
    storageObject: Storage
): [T, Dispatch<SetStateAction<T>>] {

    const [value, setValue] = useState<T>(() => {
        const jsonValue = storageObject.getItem(key)

        if (jsonValue != null) {
            return JSON.parse(jsonValue) as T
        }

        if (initialState instanceof Function) {
            return initialState()
        } else {
            return initialState
        }
    })

    useEffect(() => {
        if (value === undefined) {
            storageObject.removeItem(key)
            return
        }
        storageObject.setItem(key, JSON.stringify(value))
    }, [key, value, storageObject])

    return [value, setValue]
}

export function useFetch<T, R = unknown>(
    url: RequestInfo | URL,
    options?: {
        request?: RequestInit,
        parse?: (response: Response) => T | Promise<T>;
        initialState?: T | (() => T),
        execute?: boolean;
        onError?: (error: unknown) => void;
        onData?: (data: T) => void;
        onWillExecute?: (input: RequestInfo | URL, init?: RequestInit) => void;
    },
): AsyncState<T> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<T, R>;
};
export function useFetch<V, T, R = unknown>(
    url: RequestInfo | URL,
    options: {
        request?: RequestInit,
        parse?: (response: Response) => V | Promise<V>;
        map: (result: V) => T | Promise<T>;
        initialState?: T | (() => T),
        execute?: boolean;
        onError?: (error: unknown) => void;
        onData?: (data: T) => void;
        onWillExecute?: (input: RequestInfo | URL, init?: RequestInit) => void;
    },
): AsyncState<T> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<T, R>;
};
export function useFetch<V, T, R = unknown>(
    url: RequestInfo | URL,
    options?: {
        request?: RequestInit,
        parse?: (response: Response) => V | Promise<V>;
        map?: (result: V) => T | Promise<T>;
        initialState?: T | V | (() => T | V),
        execute?: boolean;
        onError?: (error: unknown) => void;
        onData?: (data: T | V) => void;
        onWillExecute?: (input: RequestInfo | URL, init?: RequestInit) => void;
    },
): AsyncState<T | V> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<T | V, R>;
} {
    const abortable = useRef<AbortController>();

    return useCachedPromise(
        async (inputParam: RequestInfo | URL): Promise<T | V> => {
            const response = await fetch(inputParam, { ...options?.request, signal: abortable.current?.signal });

            if (options?.parse) {
                const parsed: V = await options?.parse(response)

                if (options?.map) {
                    return options?.map(parsed)
                } else {
                    return parsed
                }
            } else {
                const content = response.headers.get("content-type");
                if (!content || !content.includes("application/json")) {
                    throw new Error("Content-Type is not 'application/json', please specify custom options.parse")
                }

                const parsed: V = await response.json()

                if (options?.map) {
                    return options?.map(parsed)
                } else {
                    return parsed
                }
            }
        },
        [url],
        {
            initialState: options?.initialState,
            abortable,
            execute: options?.execute,
            onError: options?.onError,
            onData: options?.onData,
            onWillExecute: options?.onWillExecute,
        }
    )
}
