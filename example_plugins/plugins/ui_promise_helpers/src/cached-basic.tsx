import { Detail } from "@project-gauntlet/api/components";
import { ReactElement } from "react";
import { useCachedPromise } from "@project-gauntlet/api/hooks";

export default function UseCachedPromiseBasic(): ReactElement {
    const { data, error, isLoading } = useCachedPromise(
        async (_data) => await wait(),
        ["default"]
    );

    printState(data, error, isLoading)

    return (
        <Detail isLoading={isLoading}>
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
