import React, { ReactElement } from "react";
import { Action, ActionPanel, IconAccessory, Icons, List, TextAccessory } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <List
            actions={
                <ActionPanel>
                    <Action label="Select" onAction={() => {}}/>
                </ActionPanel>
            }
        >
            <List.Item
                id="adarian"
                title="Adarian"
                icon={Icons.Checkmark}
                accessories={[
                    <TextAccessory text="Skin Color: Pink"/>,
                    <TextAccessory text="Eye Color: Black"/>,
                ]}
            />
            <List.Item
                id="aruzan"
                title="Aruzan"
                subtitle="Aruza"
                icon={Icons.Circle}
                accessories={[
                    <TextAccessory text="Skin Color: Blue"/>,
                    <TextAccessory text="Eye Color: Blue"/>,
                ]}
            />
            <List.Item
                id="blutopian"
                title="Blutopian"
                subtitle="Blutopia"
                icon={Icons.Circle}
                accessories={[
                    <TextAccessory text="Skin Color: Gray"/>,
                    <TextAccessory text="Eye Color: Brown"/>,
                ]}
            />
            <List.Item
                id="caphex"
                title="Caphex"
                subtitle="Caphexdis"
                icon={Icons.Circle}
                accessories={[
                    <TextAccessory text="Heir Color: Gray"/>,
                    <TextAccessory text="Eye Color: Brown"/>,
                ]}
            />
            <List.Item
                id="condluran"
                title="Condluran"
                icon={Icons.Checkmark}
                accessories={[
                    <TextAccessory text="Skin Color: Light"/>,
                    <TextAccessory text="Eye Color: Black"/>,
                ]}
            />
            <List.Item
                id="frozian"
                title="Frozian"
                subtitle="Froz"
                icon={Icons.Circle}
                accessories={[
                    <IconAccessory icon={Icons.Snowflake}/>,
                    <TextAccessory text="Heir Color: Brown"/>,
                    <TextAccessory text="Eye Color: Black"/>,
                ]}
            />
            <List.Item
                id="evereni"
                title="Evereni"
                subtitle="Everon"
                icon={Icons.Circle}
                accessories={[
                    <TextAccessory text="Skin color: Slate-gray"/>,
                    <TextAccessory text="Eye Color: Black"/>,
                ]}
            />
            <List.Item
                id="ezaraa"
                title="Ezaraa"
                subtitle="Ezaraa"
                icon={Icons.Checkmark}
            />
        </List>
    )
}
