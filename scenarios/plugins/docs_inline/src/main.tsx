import { ReactElement } from "react";
import { Inline, Icons } from "@project-gauntlet/api/components";

export default function Main(props: { text: string }): ReactElement {
    const type = props.text;

    switch (type) {
        case "separator": {
            return Separator()
        }
        case "three-sections": {
            return ThreeSection()
        }
        case "two-sections": {
            return TwoSection()
        }
    }

    throw new Error("unknown type")
}

function Separator() {
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

function ThreeSection() {
    return (
        <Inline>
            <Inline.Left>
                <Inline.Left.Paragraph>
                    Left
                </Inline.Left.Paragraph>
            </Inline.Left>
            <Inline.Center>
                <Inline.Center.Paragraph>
                    Center
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

function TwoSection() {
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