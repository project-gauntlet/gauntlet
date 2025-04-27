import { Detail } from "@project-gauntlet/api/components";
import { ReactElement } from "react";
import { usePromise } from "@project-gauntlet/api/hooks";

export default function UsePromiseExecuteFalse(): ReactElement {
    const { data, error, isLoading } = usePromise(
        async (_data) => await wait(),
        ["default"],
        {
            execute: false
        }
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
