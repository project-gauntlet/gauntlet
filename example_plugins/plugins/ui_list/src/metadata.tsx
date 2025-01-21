import React, { ReactElement } from "react";
import { Action, ActionPanel, List } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <List
            actions={
                <ActionPanel>
                    <Action label="Select" onAction={() => {}}/>
                </ActionPanel>
            }
        >
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
            </List.Detail>
        </List>
    )
}
