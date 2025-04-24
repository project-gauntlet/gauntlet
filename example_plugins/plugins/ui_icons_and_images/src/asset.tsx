import { ReactElement } from "react";
import { Detail } from "@project-gauntlet/api/components";

export default function Example(): ReactElement {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Image source={{ asset: "logo.png" }}/>
            </Detail.Content>
        </Detail>
    )
}
