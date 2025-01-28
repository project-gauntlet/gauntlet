import { ReactElement } from "react";
import { Inline, Icons, ActionPanel, Action } from "@project-gauntlet/api/components";

export default function Main({ text }: { text: string }): ReactElement | null {

    if (text != "1 + 2") {
        return null
    }

    return (
        <Inline
            actions={
                <ActionPanel>
                    <Action label="Copy" onAction={() => {/*  */}}/>
                </ActionPanel>
            }
        >
            <Inline.Left>
                <Inline.Left.Paragraph>
                    1 + 2
                </Inline.Left.Paragraph>
            </Inline.Left>
            <Inline.Separator icon={Icons.ArrowRight}/>
            <Inline.Right>
                <Inline.Right.Paragraph>
                    3
                </Inline.Right.Paragraph>
            </Inline.Right>
        </Inline>
    )
}
