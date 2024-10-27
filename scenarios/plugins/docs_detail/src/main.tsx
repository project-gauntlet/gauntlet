import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        // docs-code-segment:start
        <Detail>
            <Detail.Metadata>
                <Detail.Metadata.Value label={"Designation"}>Sentient</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Classification"}>Humanoid</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Homeworld"}>Ezaraa</Detail.Metadata.Value>
                <Detail.Metadata.Value label={"Diet"}>Carnivorous</Detail.Metadata.Value>
                <Detail.Metadata.TagList label={"Appearances"}>
                    <Detail.Metadata.TagList.Item>The Screaming Citadel 1</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 9</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 10</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 11</Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item>Doctor Aphra (2016) 12</Detail.Metadata.TagList.Item>
                </Detail.Metadata.TagList>
            </Detail.Metadata>
            <Detail.Content>
                <Detail.Content.Paragraph>
                    The Ezaraa were a species of warmongering carnivorous sentients that were native to the the planet Ezaraa.
                    They intended to overthrow the Galactic Empire, only to replace it with their own dominion and feed on the other species, which they deemed as lesser to them.
                    To arm their revolution, the dominion sent Ezaraa to take advantage of opportunities such as the Auction of Rur.
                </Detail.Content.Paragraph>
                <Detail.Content.H4>
                    Society and culture
                </Detail.Content.H4>
                <Detail.Content.CodeBlock>
                    "Bring the Dominion of the Ezaraa across the stars! And consume the flesh of all the lesser species!"
                </Detail.Content.CodeBlock>
                <Detail.Content.Paragraph>
                    ―An Ezaraa, to Luke Skywalker
                </Detail.Content.Paragraph>
            </Detail.Content>
        </Detail>
        // docs-code-segment:end
    )
}
