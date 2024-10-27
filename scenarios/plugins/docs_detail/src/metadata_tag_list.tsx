import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        // docs-code-segment:start
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.TagList label={"Appearances"}>
                    <Detail.Metadata.TagList.Item>The Screaming Citadel 1</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 9</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 10</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 11</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 12</Detail.Metadata.TagList.Item>
                </Detail.Metadata.TagList>
            </Detail.Metadata>
        </Detail>
        // docs-code-segment:end
    )
}
