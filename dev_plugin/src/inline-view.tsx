import { Content, Inline } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function InlineView(props: { text: string }): ReactNode | undefined {
    const text = props.text;
    if (!text.startsWith("inline")) {
        return undefined
    }

    return (
        <Inline>
            <Inline.Left>
                <Content.Paragraph>
                    Testing inline view left {text}
                </Content.Paragraph>
            </Inline.Left>
            <Inline.Separator/>
            <Inline.Right>
                <Content.Paragraph>
                    Testing inline view right
                </Content.Paragraph>
            </Inline.Right>
        </Inline>
    )
}
