import { ReactNode, useRef, useId, useState, useCallback, useEffect, MutableRefObject, Dispatch, SetStateAction } from 'react';
// @ts-ignore TODO how to add declaration for this?
import { useGauntletContext } from "gauntlet:renderer";

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

export type AsyncStateInitial = {
    isLoading: boolean; // false if options.execute is false, otherwise true
    error?: undefined;
    data?: undefined;
};
export type AsyncStateLoading<T> = {
    isLoading: true;
    error?: unknown | undefined;
    data?: T;
};
export type AsyncStateError = {
    isLoading: false;
    error: unknown;
    data?: undefined;
};
export type AsyncStateSuccess<T> = {
    isLoading: false;
    error?: undefined;
    data: T;
};

export type AsyncState<T> = AsyncStateInitial | AsyncStateLoading<T> | AsyncStateError | AsyncStateSuccess<T>;

export type MutatePromiseFn<T, R> = (
    asyncUpdate: Promise<R>,
    options?: {
        optimisticUpdate?: (data: T | undefined) => T; // undefined, if options.execute is false and function was never called, needs to be pure
        rollbackOnError?: boolean | ((data: T | undefined) => T); // only used if optimisticUpdate is specified, needs to be pure
        shouldRevalidateAfter?: boolean; // only matters for successful updates
    },
) => Promise<R>;

export type UsePromiseOptions<T extends (...args: any[]) => Promise<any>> = {
    abortable?: MutableRefObject<AbortController | undefined>;
    execute?: boolean;
    onError?: (error: unknown) => void;
    onData?: (data: Awaited<ReturnType<T>>) => void;
    onWillExecute?: (...args: Parameters<T>) => void;
}

export function usePromise<T extends (...args: any[]) => Promise<any>, R>(
    fn: T,
    args?: Parameters<T>,
    options?: UsePromiseOptions<T>,
): AsyncState<Awaited<ReturnType<T>>> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<Awaited<ReturnType<T>>, R>;
} {
    const execute = options?.execute !== false; // execute by default

    const [state, setState] = useState<AsyncState<Awaited<ReturnType<T>>>>({ isLoading: execute });

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

export function useCachedPromise<T extends (...args: any[]) => Promise<any>, R>(
    fn: T,
    args?: Parameters<T>,
    options?: UsePromiseOptions<T> & { initialState?: Awaited<ReturnType<T>> | (() => Awaited<ReturnType<T>>) },
): AsyncState<Awaited<ReturnType<T>>> & {
    revalidate: () => void;
    mutate: MutatePromiseFn<Awaited<ReturnType<T>>, R>;
} {
    const execute = options?.execute !== false; // execute by default

    const id = useId();

    const { entrypointId }: { entrypointId: () => string } = useGauntletContext();

    // same store is fetched and updated between command runs
    const [state, setState] = useCache<AsyncState<Awaited<ReturnType<T>>>>("useCachedPromise" + entrypointId() + id, () => {
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

function usePromiseInternal<T extends (...args: any[]) => Promise<any>, R>(
    fn: T,
    state: AsyncState<Awaited<ReturnType<T>>>,
    setState: Dispatch<SetStateAction<AsyncState<Awaited<ReturnType<T>>>>>,
    args: Parameters<T>,
    execute: boolean,
    abortable?: MutableRefObject<AbortController | undefined>,
    onError?: (error: unknown) => void,
    onData?: (data: Awaited<ReturnType<T>>) => void,
    onWillExecute?: (...args: Parameters<T>) => void,
): AsyncState<Awaited<ReturnType<T>>> & {
    revalidate: () => void; // will execute even if options.execute is false
    mutate: MutatePromiseFn<Awaited<ReturnType<T>>, R>; // will execute even if options.execute is false
} {

    const promiseRef = useRef<Promise<any>>();

    useEffect(() => {
        return () => {
            abortable?.current?.abort();
        };
    }, [abortable]);

    const callback = useCallback(async (...args: Parameters<T>): Promise<void> => {
        if (abortable) {
            abortable.current?.abort();
            abortable.current = new AbortController()
        }

        onWillExecute?.(...args);

        const promise = fn(...args);

        setState(prevState => ({ ...prevState, isLoading: true }));

        promiseRef.current = promise;

        let promiseResult: Awaited<ReturnType<T>>;
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
                optimisticUpdate?: (data: Awaited<ReturnType<T>> | undefined) => Awaited<ReturnType<T>>;
                rollbackOnError?: boolean | ((data: Awaited<ReturnType<T>> | undefined) => Awaited<ReturnType<T>>);
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
                            if (prevData === undefined) {
                                setState({ data: prevData, isLoading: false })
                            } else {
                                setState({ data: prevData, isLoading: false })
                            }
                            break;
                        }
                        case "boolean": {
                            if (rollbackOnError) {
                                if (prevData === undefined) {
                                    setState({ data: prevData, isLoading: false })
                                } else {
                                    setState({ data: prevData, isLoading: false })
                                }
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
            return storageObject.removeItem(key)
        }
        storageObject.setItem(key, JSON.stringify(value))
    }, [key, value, storageObject])

    return [value, setValue]
}
