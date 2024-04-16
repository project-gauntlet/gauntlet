import { Content, Icons, Inline } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

// @ts-expect-error
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi: InternalApi = denoCore.ops;

interface InternalApi {
    run_numbat(input: string): { left: string, right: string }
}

export default function Default(props: { text: string }): ReactNode | undefined {
    const text = props.text;

    try {
        const { left, right } = InternalApi.run_numbat(text);

        if (left == right) {
            return undefined
        }

        return (
            <Inline>
                <Inline.Left>
                    <Content.Paragraph>
                        {left}
                    </Content.Paragraph>
                </Inline.Left>
                <Inline.Separator icon={Icons.ArrowRight}/>
                <Inline.Right>
                    <Content.Paragraph>
                        {right}
                    </Content.Paragraph>
                </Inline.Right>
            </Inline>
        )
    } catch (e) {
        return undefined
    }
}
