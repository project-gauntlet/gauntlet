import { ReactElement } from "react";
import { Inline, Icons } from "@project-gauntlet/api/components";

export default function Main({ text }: { text: string }): ReactElement | null {

    if (text != "example") {
        return null
    }

    return (
        <Inline>
            <Inline.Left>
                <Inline.Left.Paragraph>
                    Left
                </Inline.Left.Paragraph>
            </Inline.Left>
            <Inline.Separator icon={Icons.ArrowCounterClockwise}/>
            <Inline.Right>
                <Inline.Right.Paragraph>
                    Right
                </Inline.Right.Paragraph>
            </Inline.Right>
        </Inline>
    )
}
