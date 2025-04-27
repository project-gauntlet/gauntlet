import { ActionPanel, Detail } from "@project-gauntlet/api/components";
import { ReactElement } from "react";
import { usePromise } from "@project-gauntlet/api/hooks";

export default function UsePromiseRevalidate(): ReactElement {
    const { data, error, isLoading, revalidate } = usePromise(
        async (_data) => await wait(),
        ["default"]
    );

    printState(data, error, isLoading)

    return (
        <Detail
            actions={
                <ActionPanel>
                    <ActionPanel.Action label="Revalidate" onAction={() => revalidate()}/>
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

async function wait(): Promise<string> {
    return new Promise(resolve => {
        setTimeout(() => resolve(`${Math.random()}`), 2000);
    })
}

function printState(data: any, error: unknown, isLoading: boolean) {
    console.log("")
    console.dir(isLoading)
    console.dir(data)
    console.dir(error)
}
