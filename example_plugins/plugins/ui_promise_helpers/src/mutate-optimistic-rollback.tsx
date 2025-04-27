import { ActionPanel, Detail } from "@project-gauntlet/api/components";
import { ReactElement } from "react";
import { usePromise } from "@project-gauntlet/api/hooks";

export default function UsePromiseMutateOptimisticRollback(): ReactElement {
    const { data, error, isLoading, mutate } = usePromise(
        async (_data) => await wait(),
        ["default"]
    );

    printState(data, error, isLoading)

    return (
        <Detail
            actions={
                <ActionPanel>
                    <ActionPanel.Action
                        label="Mutate"
                        onAction={async () => {
                            mutate(
                                wait(true),
                                {
                                    optimisticUpdate: oldData => oldData + " optimistic",
                                    rollbackOnError: oldData => oldData + " failed",
                                }
                            )
                        }}
                    />
                </ActionPanel>
            }
            isLoading={isLoading}
        >
            <Detail.Content>
                <Detail.Content.Paragraph>
                    {"Data " + data}
                </Detail.Content.Paragraph>
            </Detail.Content>
        </Detail>
    )
}

async function wait(error: boolean = false): Promise<string> {
    return new Promise((resolve, reject) => {
        setTimeout(() => {
            const value = `${Math.random()}`;
            if (error) {
                reject(value)
            } else {
                resolve(value)
            }
        }, 2000);
    })
}

function printState(data: any, error: unknown, isLoading: boolean) {
    console.log("")
    console.dir(isLoading)
    console.dir(data)
    console.dir(error)
}
