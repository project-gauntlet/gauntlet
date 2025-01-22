import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function SectionExample(): ReactElement {
    return (
        <List>
            <List.Item id="star-wars" title="Star Wars"/>
            <List.Section title="Species">
                <List.Item id="frozian" title="Frozian"/>
                <List.Item id="evereni" title="Evereni"/>
                <List.Item id="ezaraa" title="Ezaraa"/>
            </List.Section>
            <List.Section title="Planets">
                <List.Item id="ryloth" title="Ryloth"/>
                <List.Item id="tatooine" title="Tatooine"/>
                <List.Item id="dagobah" title="Dagobah"/>
                <List.Item id="coruscant" title="Coruscant"/>
            </List.Section>
        </List>
    )
}
