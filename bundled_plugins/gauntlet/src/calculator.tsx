import { Action, ActionPanel, Content, Icons, Inline } from "@project-gauntlet/api/components";
import { ReactNode } from "react";
import { Clipboard, showHud } from "@project-gauntlet/api/helpers";

// @ts-expect-error
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi: InternalApi = denoCore.ops;

interface InternalApi {
    run_numbat(input: string): { left: string, right: string }
}

export default function Calculator(props: { text: string }): ReactNode | undefined {
    const text = props.text;

    if (text.length < 3) {
        return undefined
    }

    let result;

    try {
         result = InternalApi.run_numbat(text);
    } catch (e) {
        // this view is executed on every key press in main search bar
        // when numbat run fails it means expression is not valid so we return here and do not show inline view
        return undefined
    }

    const { left, right } = result;

    if (left == right) {
        return undefined
    }

    return (
        <Inline
            actions={
                <ActionPanel>
                    <Action
                        label={"Copy result"}
                        onAction={async () => {
                            await Clipboard.writeText(right)
                            showHud("Result copied")
                        }}
                    />
                </ActionPanel>
            }
        >
            <Inline.Left>
                <Content.H3>
                    {left}
                </Content.H3>
            </Inline.Left>
            <Inline.Separator icon={Icons.ArrowRight}/>
            <Inline.Right>
                <Content.H3>
                    {right}
                </Content.H3>
            </Inline.Right>
        </Inline>
    )
}
