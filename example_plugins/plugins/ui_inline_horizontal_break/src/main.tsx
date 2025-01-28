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
                    Above
                </Inline.Center.Paragraph>
                <Inline.Center.HorizontalBreak/>
                <Inline.Center.Paragraph>
                    Below
                </Inline.Center.Paragraph>
            </Inline.Center>
        </Inline>
    )
}
