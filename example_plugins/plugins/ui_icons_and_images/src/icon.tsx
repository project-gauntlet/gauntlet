import { ReactElement } from "react";
import { Detail, Icons } from "@project-gauntlet/api/components";

export default function Example(): ReactElement {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Image source={Icons.Eject}/>
            </Detail.Content>
        </Detail>
    )
}
