import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";

export default function Metadata(): ReactNode {
    return (
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Value label={"Name"}>Ezaraa</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Designation"}>Sentient</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Classification"}>Humanoid</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Homeworld"}>Ezaraa</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Diet"}>Carnivorous</Detail.Metadata.Value>
                <Detail.Metadata.Link label={"Wiki"} href="https://starwars.fandom.com/wiki/Ezaraa">Link</Detail.Metadata.Link>
                <Detail.Metadata.TagList label={"Appearances"}>
                    <Detail.Metadata.TagList.Item>The Screaming Citadel 1</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 9</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 10</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 11</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 12</Detail.Metadata.TagList.Item>
                </Detail.Metadata.TagList>
            </Detail.Metadata>
        </Detail>
    )
}
