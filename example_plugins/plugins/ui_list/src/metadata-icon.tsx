import { ReactElement } from "react";
import { Icons, List } from "@project-gauntlet/api/components";

export default function MetadataIconExample(): ReactElement {
    return (
        <List>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Detail>
                <List.Detail.Metadata>
                    <List.Detail.Metadata.Icon label="Canon" icon={Icons.Checkmark}/>
                    <List.Detail.Metadata.Icon label="Legends" icon={Icons.XMark}/>
                </List.Detail.Metadata>
            </List.Detail>
        </List>
    )
}
