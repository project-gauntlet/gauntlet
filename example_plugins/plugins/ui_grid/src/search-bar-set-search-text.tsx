import { ReactElement, useState } from "react";
import { Action, ActionPanel, Grid } from "@project-gauntlet/api/components";

export default function SearchBarSetSearchTextExample(): ReactElement {
    const [searchText, setSearchText] = useState<string>("");

    return (
        <Grid
            actions={
                <ActionPanel>
                    <Action
                        label="Set value"
                        onAction={(id) => {
                            // This will be the value in search bar when
                            // enter is pressed while item is not focused
                            const unfocused = "item-not-unfocused";

                            setSearchText(id || unfocused)
                        }}
                    />
                </ActionPanel>
            }
        >
            <Grid.SearchBar value={searchText} onChange={setSearchText}/>
            {/* This id will be the value in search bar when clicked or press enter when item is focused */}
            <Grid.Item id="item-focused">
                <Grid.Item.Content>
                    <Grid.Item.Content.Paragraph>
                        Click me!
                    </Grid.Item.Content.Paragraph>
                </Grid.Item.Content>
            </Grid.Item>
        </Grid>
    )
}
