import { Detail, Icons } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Icon label={"Media"} icon={Icons.Film}/>
            </Detail.Metadata>
        </Detail>
    )
}
