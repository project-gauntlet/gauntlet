import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.CodeBlock>
                    TODO
                    &lt;Detail.Content.Image source=&#123;&#123; data: ... &#125;&#125;/&gt;
                </Detail.Content.CodeBlock>
            </Detail.Content>
        </Detail>
    )
}
