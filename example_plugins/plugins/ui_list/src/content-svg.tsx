import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

const svgUrl = "https://upload.wikimedia.org/wikipedia/commons/8/84/Example.svg"

export default function ContentSvgExample(): ReactElement {
    return (
        <List>
            <List.Item id="svg" title="Svg Example"/>
            <List.Detail>
                <List.Detail.Content>
                    <List.Detail.Content.Svg source={{ url: svgUrl }}/>
                </List.Detail.Content>
            </List.Detail>
        </List>
    )
}
