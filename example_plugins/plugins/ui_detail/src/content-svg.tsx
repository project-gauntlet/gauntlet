import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";

const svgUrl = "https://upload.wikimedia.org/wikipedia/commons/8/84/Example.svg"

export default function ContentSvg(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Svg source={{ url: svgUrl }}/>
            </Detail.Content>
        </Detail>
    )
}
