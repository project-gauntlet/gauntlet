import { ReactElement, useState } from "react";
import { Action, ActionPanel, Grid } from "@project-gauntlet/api/components";

export default function SearchBarSetSearchTextExample(): ReactElement {
    const [searchText, setSearchText] = useState<string | undefined>("");

    return (
        <Grid
            actions={
                <ActionPanel>
                    <Action label="Set value" onAction={(id) => setSearchText(id)}/>
                </ActionPanel>
            }
        >
            <Grid.SearchBar value={searchText} onChange={setSearchText}/>
            <Grid.Item id="This will be the value in search bar">
                <Grid.Item.Content>
                    <Grid.Item.Content.Paragraph>
                        Click me!
                    </Grid.Item.Content.Paragraph>
                </Grid.Item.Content>
            </Grid.Item>
        </Grid>
    )
}
