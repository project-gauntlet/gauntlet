import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";
import { showHud } from "@project-gauntlet/api/helpers";

const items = [
    "The Screaming Citadel 1",
    "Doctor Aphra (2016) 9",
    "Doctor Aphra (2016) 10",
    "Doctor Aphra (2016) 11",
    "Doctor Aphra (2016) 12"
]

export default function MetadataTagList(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.TagList label={"Appearances"}>
                    {
                        items.map(value => {
                            return (
                                <Detail.Metadata.TagList.Item onClick={() => showHud(value)}>
                                    {value}
                                </Detail.Metadata.TagList.Item>
                            )
                        })
                    }
                </Detail.Metadata.TagList>
            </Detail.Metadata>
        </Detail>
    )
}
