import { ReactElement } from "react";
import { Inline } from "@project-gauntlet/api/components";

export default function Main({ text }: { text: string }): ReactElement | null {

    if (text != "example") {
        return null
    }

    return (
        <Inline>
            <Inline.Center>
                <Inline.Center.H3>
                    Header Text
                </Inline.Center.H3>
            </Inline.Center>
        </Inline>
    )
}
