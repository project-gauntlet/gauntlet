import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";

export default function MetadataValue(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Value label={"Voiced By"}>David Tennant</Detail.Metadata.Value>
            </Detail.Metadata>
        </Detail>
    )
}
