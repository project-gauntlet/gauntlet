import { Action, ActionPanel, Content, Icons, Inline } from "@project-gauntlet/api/components";
import { ReactNode } from "react";
import { Clipboard } from "@project-gauntlet/api/helpers";

export default function InlineView(props: { text: string }): ReactNode | undefined {
    const text = props.text;
    if (!text.startsWith("inline")) {
        return undefined
    }

    return (
        <Inline
            actions={
                <ActionPanel>
                    <Action
                        label={"Copy content"}
                        onAction={async () => {
                            console.log("action test 1")
                            await Clipboard.writeText("Test Content")
                        }}
                    />
                    <Action
                        label={"Test 2"}
                        onAction={() => {
                            console.log("action test 2")
                        }}
                    />
                    <Action
                        id="testInlineAction"
                        label={"Test 3"}
                        onAction={() => {
                            console.log("action test 3")
                        }}
                    />
                </ActionPanel>
            }
        >
            <Inline.Left>
                <Content.Paragraph>
                    Testing inline view left {text}
                </Content.Paragraph>
            </Inline.Left>
            <Inline.Separator/>
            <Inline.Center>
                <Content.Paragraph>
                    Testing inline view center
                </Content.Paragraph>
            </Inline.Center>
            <Inline.Separator icon={Icons.ArrowRight}/>
            <Inline.Right>
                <Content.Paragraph>
                    Testing inline view right
                </Content.Paragraph>
            </Inline.Right>
        </Inline>
    )
}
