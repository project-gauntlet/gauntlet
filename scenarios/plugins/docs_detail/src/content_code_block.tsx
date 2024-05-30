import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.CodeBlock>
                    "Bring the Dominion of the Ezaraa across the stars! And consume the flesh of all the lesser species!"
                </Detail.Content.CodeBlock>
            </Detail.Content>
        </Detail>
    )
}
