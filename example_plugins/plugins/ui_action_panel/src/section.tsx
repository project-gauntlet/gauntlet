import { ReactElement } from "react";
import { ActionPanel, List } from "@project-gauntlet/api/components";

export default function View(): ReactElement {
    return (
        <List actions={
            <ActionPanel title="Action Panel Title">
                <ActionPanel.Action
                    label="Primary action"
                    onAction={id => console.log(`Primary action for item with id '${id}' was executed`)}
                />
                <ActionPanel.Section title="Section title">
                    <ActionPanel.Action
                        label="Secondary action"
                        onAction={id => console.log(`Secondary action for item with id '${id}' was executed`)}
                    />
                    <ActionPanel.Action
                        label="Another action"
                        onAction={id => console.log(`Another action for item with id '${id}' was executed`)}
                    />
                </ActionPanel.Section>
                <ActionPanel.Section title="Another Section title">
                    <ActionPanel.Action
                        label="Yet another action"
                        onAction={id => console.log(`Yet another action for item with id '${id}' was executed`)}
                    />
                </ActionPanel.Section>
            </ActionPanel>
        }>
            <List.Item id={"item"} title={"Item"}/>
        </List>
    )
}
