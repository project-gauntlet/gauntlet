import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        // docs-code-segment:start
        <List>
            <List.Item title="Star Wars"/>
            <List.Section title="Species">
                <List.Item title="Frozian"/>
                <List.Item title="Evereni"/>
                <List.Item title="Ezaraa"/>
            </List.Section>
            <List.Section title="Planets">
                <List.Item title="Ryloth"/>
                <List.Item title="Tatooine"/>
                <List.Item title="Dagobah"/>
                <List.Item title="Coruscant"/>
            </List.Section>
        </List>
        // docs-code-segment:end
    )
}
