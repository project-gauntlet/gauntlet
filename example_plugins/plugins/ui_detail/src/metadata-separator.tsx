import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";

export default function MetadataSeparator(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Value label={"Name"}>Ezaraa</Detail.Metadata.Value>
                <Detail.Metadata.Separator/>
                <Detail.Metadata.Value label={"Designation"}>Sentient</Detail.Metadata.Value>
            </Detail.Metadata>
        </Detail>
    )
}
