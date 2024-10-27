import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        // docs-code-segment:start
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Value label={"Voiced By"}>David Tennant</Detail.Metadata.Value>
            </Detail.Metadata>
        </Detail>
        // docs-code-segment:end
    )
}
