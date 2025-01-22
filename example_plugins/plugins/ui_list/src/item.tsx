import { ReactElement } from "react";
import { IconAccessory, Icons, List, TextAccessory } from "@project-gauntlet/api/components";

export default function ItemExample(): ReactElement {
    return (
        <List>
            <List.Item id="frozian" title="Frozian" accessories={[<IconAccessory icon={Icons.Snowflake}/>]}/>
            <List.Item id="ezaraa" title="Ezaraa" accessories={[<TextAccessory text="Telepathic"/>]}/>
            <List.Item id="blutopian" title="Blutopian" subtitle="Rogue One: A Star Wars Story"/>
            <List.Item id="caphex" title="Caphex" icon={Icons.Circle}/>
        </List>
    )
}
