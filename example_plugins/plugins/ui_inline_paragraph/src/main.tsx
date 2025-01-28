import { ReactElement } from "react";
import { Inline } from "@project-gauntlet/api/components";

export default function Main({ text }: { text: string }): ReactElement | null {

    if (text != "example") {
        return null
    }

    return (
        <Inline>
            <Inline.Center>
                <Inline.Center.Paragraph>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod
                </Inline.Center.Paragraph>
            </Inline.Center>
        </Inline>
    )
}
