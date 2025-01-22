import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function MetadataLinkExample(): ReactElement {
    return (
        <List>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Detail>
                <List.Detail.Metadata>
                    <List.Detail.Metadata.Link label="Wiki" href="https://starwars.fandom.com/wiki/Ezaraa"/>
                </List.Detail.Metadata>
            </List.Detail>
        </List>
    )
}
