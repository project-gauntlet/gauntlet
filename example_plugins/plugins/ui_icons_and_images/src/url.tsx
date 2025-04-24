import { ReactElement } from "react";
import { Detail } from "@project-gauntlet/api/components";

export default function Example(): ReactElement {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Image source={{ url: "https://github.com/project-gauntlet/gauntlet/blob/main/docs/logo.png?raw=true" }}/>
            </Detail.Content>
        </Detail>
    )
}
