import { ReactElement } from "react";
import { Icons, List } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <List>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Detail>
                <List.Detail.Metadata>
                    <List.Detail.Metadata.Value label="Designation">Sentient</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Value label="Classification">Humanoid</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Separator/>
                    <List.Detail.Metadata.Value label="Homeworld">Ezaraa</List.Detail.Metadata.Value>
                    <List.Detail.Metadata.Value label="Diet">Carnivorous</List.Detail.Metadata.Value>
                </List.Detail.Metadata>
            </List.Detail>
        </List>
    )
}
