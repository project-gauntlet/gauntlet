import { Detail } from "@project-gauntlet/api/components";
import { ReactElement } from "react";
import { useFetch } from "@project-gauntlet/api/hooks";

export default function UseFetchBasic(): ReactElement {
    interface GithubLatestRelease {
        // ...
    }

    const { data, error, isLoading } = useFetch<GithubLatestRelease>(
        "https://api.github.com/repos/project-gauntlet/gauntlet/releases/latest"
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

function printState(data: any, error: unknown, isLoading: boolean) {
    console.log("")
    console.dir(isLoading)
    console.dir(data)
    console.dir(error)
}
