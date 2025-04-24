import { ReactElement } from "react";
import { ActionPanel, List } from "@project-gauntlet/api/components";

export default function View(): ReactElement {
    return (
        <List actions={
            <ActionPanel title={"Title"}>
                <ActionPanel.Action
                    label={"Primary action"}
                    onAction={id => {
                        console.log(`Primary action for item with id '${id}' was executed`)
                        // returning object with close property set to true will close the window
                        return { close: true }
                    }}
                />
                <ActionPanel.Action
                    label={"Secondary action"}
                    onAction={id => console.log(`Secondary action for item with id '${id}' was executed`)}
                />
                <ActionPanel.Action
                    id="actionId"
                    label={"Another action"}
                    onAction={id => console.log(`Another action for item with id '${id}' was executed`)}
                />
            </ActionPanel>
        }>
            <List.Item id={"item"} title={"Item"}/>
        </List>
    )
}

