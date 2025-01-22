import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function MetadataValueExample(): ReactElement {
    return (
        <List>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Detail>
                <List.Detail.Metadata>
                    <List.Detail.Metadata.Value label="Designation">Sentient</List.Detail.Metadata.Value>
                </List.Detail.Metadata>
            </List.Detail>
        </List>
    )
}
