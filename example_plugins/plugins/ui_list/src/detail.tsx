import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function DetailExample(): ReactElement {
    return (
        <List>
            <List.Item id="adarian" title="Adarian"/>
            <List.Item id="aruzan" title="Aruzan"/>
            <List.Item id="blutopian" title="Blutopian"/>
            <List.Item id="caphex" title="Caphex"/>
            <List.Item id="condluran" title="Condluran"/>
            <List.Item id="frozian" title="Frozian"/>
            <List.Item id="evereni" title="Evereni"/>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Item id="houk" title="Houk"/>
            <List.Item id="inleshat" title="Inleshat"/>
            <List.Detail>
                <List.Detail.Metadata>
                    <List.Detail.Metadata.Value label={"Name"}>Ezaraa</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Value label={"Designation"}>Sentient</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Value label={"Classification"}>Humanoid</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Value label={"Homeworld"}>Ezaraa</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Value label={"Diet"}>Carnivorous</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.TagList label={"Appearances"}>
                        <List.Detail.Metadata.TagList.Item>The Screaming Citadel 1</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Doctor Aphra (2016) 9</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Doctor Aphra (2016) 10</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Doctor Aphra (2016) 11</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Doctor Aphra (2016) 12</List.Detail.Metadata.TagList.Item>
                    </List.Detail.Metadata.TagList>
                </List.Detail.Metadata>
                <List.Detail.Content>
                    <List.Detail.Content.Paragraph>
                        The Ezaraa were a species of warmongering carnivorous sentients that were native to the the planet
                        Ezaraa.
                        They intended to overthrow the Galactic Empire, only to replace it with their own dominion and feed
                        on the other species, which they deemed as lesser to them.
                        To arm their revolution, the dominion sent Ezaraa to take advantage of opportunities such as the
                        Auction of Rur.
                    </List.Detail.Content.Paragraph>
                    <List.Detail.Content.H4>
                        Society and culture
                    </List.Detail.Content.H4>
                    <List.Detail.Content.CodeBlock>
                        "Bring the Dominion of the Ezaraa across the stars! And consume the flesh of all the lesser
                        species!"
                    </List.Detail.Content.CodeBlock>
                    <List.Detail.Content.Paragraph>
                        ―An Ezaraa, to Luke Skywalker
                    </List.Detail.Content.Paragraph>
                </List.Detail.Content>
            </List.Detail>
        </List>
    )
}
