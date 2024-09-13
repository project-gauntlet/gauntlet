import { Icons, List } from "@project-gauntlet/api/components";
import React, { ReactElement, useRef } from "react";
import { useCachedPromise, useFetch, useNavigation, usePromise } from "@project-gauntlet/api/hooks";

export default function ListView(): ReactElement {
    const { pushView } = useNavigation();

    return (
        <List
            onSelectionChange={id => {
                switch (id) {
                    case "UsePromiseTestBasic": {
                        pushView(<UsePromiseTestBasic/>)
                        break;
                    }
                    case "UsePromiseTestExecuteFalse": {
                        pushView(<UsePromiseTestExecuteFalse/>)
                        break;
                    }
                    case "UsePromiseTestRevalidate": {
                        pushView(<UsePromiseTestRevalidate/>)
                        break;
                    }
                    case "UsePromiseTestAbortableRevalidate": {
                        pushView(<UsePromiseTestAbortableRevalidate/>)
                        break;
                    }
                    case "UsePromiseTestMutate": {
                        pushView(<UsePromiseTestMutate/>)
                        break;
                    }
                    case "UsePromiseTestMutateOptimistic": {
                        pushView(<UsePromiseTestMutateOptimistic/>)
                        break;
                    }
                    case "UsePromiseTestMutateOptimisticRollback": {
                        pushView(<UsePromiseTestMutateOptimisticRollback/>)
                        break;
                    }
                    case "UsePromiseTestMutateNoRevalidate": {
                        pushView(<UsePromiseTestMutateNoRevalidate/>)
                        break;
                    }
                    case "UsePromiseTestThrow": {
                        pushView(<UsePromiseTestThrow/>)
                        break;
                    }
                    case "UseCachedPromiseBasic": {
                        pushView(<UseCachedPromiseBasic/>)
                        break;
                    }
                    case "UseCachedPromiseInitialState": {
                        pushView(<UseCachedPromiseInitialState/>)
                        break;
                    }
                    case "UseFetchBasic": {
                        pushView(<UseFetchBasic/>)
                        break;
                    }
                    case "UseFetchMap": {
                        pushView(<UseFetchMap/>)
                        break;
                    }
                }
            }}
        >
            <List.Item id="UsePromiseTestBasic" title="UsePromiseTestBasic"/>
            <List.Item id="UsePromiseTestExecuteFalse" title="UsePromiseTestExecuteFalse"/>
            <List.Item id="UsePromiseTestRevalidate" title="UsePromiseTestRevalidate"/>
            <List.Item id="UsePromiseTestAbortableRevalidate" title="UsePromiseTestAbortableRevalidate"/>
            <List.Item id="UsePromiseTestMutate" title="UsePromiseTestMutate"/>
            <List.Item id="UsePromiseTestMutateOptimistic" title="UsePromiseTestMutateOptimistic"/>
            <List.Item id="UsePromiseTestMutateOptimisticRollback" title="UsePromiseTestMutateOptimisticRollback"/>
            <List.Item id="UsePromiseTestMutateNoRevalidate" title="UsePromiseTestMutateNoRevalidate"/>
            <List.Item id="UsePromiseTestThrow" title="UsePromiseTestThrow"/>
            <List.Item id="UseCachedPromiseBasic" title="UseCachedPromiseBasic"/>
            <List.Item id="UseCachedPromiseInitialState" title="UseCachedPromiseInitialState"/>
            <List.Item id="UseFetchBasic" title="UseFetchBasic"/>
            <List.Item id="UseFetchMap" title="UseFetchMap"/>
        </List>
    )
}

function UsePromiseTestBasic(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3]
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
               <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UseCachedPromiseBasic(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading } = useCachedPromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3]
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UseCachedPromiseInitialState(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading } = useCachedPromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
        {
            initialState: () => "initial"
        }
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestExecuteFalse(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
        {
            execute: false
        }
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestRevalidate(): ReactElement {
    const { popView } = useNavigation();

    const { data, error, isLoading, revalidate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView, () => revalidate())}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="run" title="Run" icon={Icons.Sun}/>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestAbortableRevalidate(): ReactElement {
    const { popView } = useNavigation();
    const abortable = useRef<AbortController>();

    const { data, error, isLoading, revalidate } = usePromise(
        async (one, two, three) => {
            await inNSec(5)
        },
        [1, 2, 3],
        {
            abortable,
        }
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView, () => revalidate())}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="run" title="Run" icon={Icons.Sun}/>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestMutate(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView, async () => {
                await mutate(inNSec(5))
            })}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="run" title="Run" icon={Icons.Sun}/>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestMutateOptimistic(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView, async () => {
                await mutate(
                    inNSec(5),
                    {
                        optimisticUpdate: data1 => data1 + " optimistic",
                    }
                )
            })}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="run" title="Run" icon={Icons.Sun}/>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestMutateOptimisticRollback(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView, async () => {
                await mutate(
                    new Promise<string>((_resolve, reject) => {
                        setTimeout(
                            () => {
                                reject("fail")
                            },
                            5 * 1000
                        );
                    }),
                    {
                        optimisticUpdate: data1 => data1 + " optimistic",
                        rollbackOnError:  data1 => data1 + " failed",
                    }
                );
            })}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="run" title={"Run " + data} icon={Icons.Sun}/>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestMutateNoRevalidate(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView, async () => {
                await mutate(
                    inNSec(5),
                    {
                        shouldRevalidateAfter: false,
                    }
                )
            })}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="run" title="Run" icon={Icons.Sun}/>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UsePromiseTestThrow(): ReactElement {
    const { popView } = useNavigation();
    const { data, error, isLoading } = usePromise(
        async (one, two, three) => {
            throw new Error("test")
        },
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UseFetchBasic(): ReactElement {
    const { popView } = useNavigation();

    interface GithubLatestRelease {

    }

    const { data, error, isLoading } = useFetch<GithubLatestRelease>(
        "https://api.github.com/repos/project-gauntlet/gauntlet/releases/latest"
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function UseFetchMap(): ReactElement {
    interface GithubLatestRelease {
        url: string
    }

    const { popView } = useNavigation();
    const { data, error, isLoading } = useFetch<GithubLatestRelease, string>(
        "https://api.github.com/repos/project-gauntlet/gauntlet/releases/latest",
        {
            map: result => result.url
        }
    );

    printState(data, error, isLoading)

    return (
        <List
            isLoading={isLoading}
            onSelectionChange={onSelectionChangeHandler(popView)}
        >
            <List.Section title={"Data " + data}>
                <List.Item id="back" title="Go Back" icon={Icons.Clipboard}/>
            </List.Section>
        </List>
    )
}

function onSelectionChangeHandler(popView: () => void, handler?: () => void): (id: string) => void {
    return (id) => {
        switch (id) {
            case "back": {
                popView()
                break;
            }
            case "run": {
                handler?.()
                break;
            }
        }
    }
}

async function inNSec(n: number): Promise<string> {
    return new Promise<string>(resolve => {
        setTimeout(
            () => {
                resolve(`Value: ${Math.random()}`)
            },
            n * 1000
        );
    })
}

function printState(data: any, error: unknown, isLoading: boolean) {
    console.log("")
    console.log("=====")
    console.dir(data)
    console.dir(error)
    console.dir(isLoading)
}