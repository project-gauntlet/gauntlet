import { Icons, List } from "@project-gauntlet/api/components";
import React, { ReactElement, useRef } from "react";
import { useCachedPromise, useFetch, useNavigation, usePromise } from "@project-gauntlet/api/hooks";

export default function ListView(): ReactElement {
    const { pushView } = useNavigation();

    return (
        <List>
            <List.Item title="UsePromiseTestBasic" onClick={() => pushView(<UsePromiseTestBasic/>)}/>
            <List.Item title="UsePromiseTestExecuteFalse" onClick={() => pushView(<UsePromiseTestExecuteFalse/>)}/>
            <List.Item title="UsePromiseTestRevalidate" onClick={() => pushView(<UsePromiseTestRevalidate/>)}/>
            <List.Item title="UsePromiseTestAbortableRevalidate" onClick={() => pushView(<UsePromiseTestAbortableRevalidate/>)}/>
            <List.Item title="UsePromiseTestMutate" onClick={() => pushView(<UsePromiseTestMutate/>)}/>
            <List.Item title="UsePromiseTestMutateOptimistic" onClick={() => pushView(<UsePromiseTestMutateOptimistic/>)}/>
            <List.Item title="UsePromiseTestMutateOptimisticRollback" onClick={() => pushView(<UsePromiseTestMutateOptimisticRollback/>)}/>
            <List.Item title="UsePromiseTestMutateNoRevalidate" onClick={() => pushView(<UsePromiseTestMutateNoRevalidate/>)}/>
            <List.Item title="UsePromiseTestThrow" onClick={() => pushView(<UsePromiseTestThrow/>)}/>
            <List.Item title="UseCachedPromiseBasic" onClick={() => pushView(<UseCachedPromiseBasic/>)}/>
            <List.Item title="UseCachedPromiseInitialState" onClick={() => pushView(<UseCachedPromiseInitialState/>)}/>
            <List.Item title="UseFetchBasic" onClick={() => pushView(<UseFetchBasic/>)}/>
            <List.Item title="UseFetchMap" onClick={() => pushView(<UseFetchMap/>)}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Run" icon={Icons.Sun} onClick={() => revalidate()}/>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Run" icon={Icons.Sun} onClick={() => revalidate()}/>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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

    const onClick = async () => {
        await mutate(inNSec(5))
    };
    return (
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Run" icon={Icons.Sun} onClick={onClick}/>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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

    const onClick = async () => {
        await mutate(
            inNSec(5),
            {
                optimisticUpdate: data1 => data1 + " optimistic",
            }
        )
    };

    return (
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Run" icon={Icons.Sun} onClick={onClick}/>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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

    const onClick = async () => {
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
    };

    return (
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Run" icon={Icons.Sun} onClick={onClick}/>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Run" icon={Icons.Sun} onClick={() => async () => {
                    await mutate(
                        inNSec(5),
                        {
                            shouldRevalidateAfter: false,
                        }
                    )
                }}/>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
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
        <List isLoading={isLoading}>
            <List.Section title={"Data " + data}>
                <List.Item title="Go Back" icon={Icons.Clipboard} onClick={() => popView()}/>
            </List.Section>
        </List>
    )
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