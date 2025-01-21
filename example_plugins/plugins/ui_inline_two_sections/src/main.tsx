import { ReactElement } from "react";
import { Inline } from "@project-gauntlet/api/components";

export default function Main({ text }: { text: string }): ReactElement {
    return (
        <Inline>
            <Inline.Left>
                <Inline.Left.Paragraph>
                    Left
                </Inline.Left.Paragraph>
            </Inline.Left>
            <Inline.Right>
                <Inline.Right.Paragraph>
                    Right
                </Inline.Right.Paragraph>
            </Inline.Right>
        </Inline>
    )
}
