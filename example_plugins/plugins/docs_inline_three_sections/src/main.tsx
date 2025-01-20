import { ReactElement } from "react";
import { Inline } from "@project-gauntlet/api/components";

export default function Main(props: { text: string }): ReactElement {
    const type = props.text;

    return (
        <Inline>
            <Inline.Left>
                <Inline.Left.Paragraph>
                    Left
                </Inline.Left.Paragraph>
            </Inline.Left>
            <Inline.Center>
                <Inline.Center.Paragraph>
                    Center: {type}
                </Inline.Center.Paragraph>
            </Inline.Center>
            <Inline.Right>
                <Inline.Right.Paragraph>
                    Right
                </Inline.Right.Paragraph>
            </Inline.Right>
        </Inline>
    )
}
