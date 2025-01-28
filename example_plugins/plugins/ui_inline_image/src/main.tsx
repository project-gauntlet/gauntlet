import { ReactElement } from "react";
import { Icons, Inline } from "@project-gauntlet/api/components";

export default function Main({ text }: { text: string }): ReactElement | null {

    if (text != "example") {
        return null
    }

    return (
        <Inline>
            <Inline.Center>
                <Inline.Center.Image source={Icons.Airplane}/>
            </Inline.Center>
        </Inline>
    )
}
