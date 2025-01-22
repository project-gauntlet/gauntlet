import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";

export default function MetadataLink(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Link label={"Wiki"} href="https://starwars.fandom.com/wiki/Ezaraa">Link</Detail.Metadata.Link>
            </Detail.Metadata>
        </Detail>
    )
}
