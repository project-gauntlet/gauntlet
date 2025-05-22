import { ReactElement } from "react";
import { Inline } from "@project-gauntlet/api/components";

const svgUrl = "https://upload.wikimedia.org/wikipedia/commons/8/84/Example.svg"

export default function Main({ text }: { text: string }): ReactElement | null {

    if (text != "example") {
        return null
    }

    return (
        <Inline>
            <Inline.Center>
                <Inline.Center.Svg source={{ url: svgUrl }}/>
            </Inline.Center>
        </Inline>
    )
}
