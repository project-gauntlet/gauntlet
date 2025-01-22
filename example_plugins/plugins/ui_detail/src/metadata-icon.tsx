import { ReactNode } from "react";
import { Detail, Icons } from "@project-gauntlet/api/components";

export default function MetadataIcon(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Icon label={"Media"} icon={Icons.Film}/>
            </Detail.Metadata>
        </Detail>
    )
}
