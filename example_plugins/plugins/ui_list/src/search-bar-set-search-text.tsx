import { ReactElement, useState } from "react";
import { Action, ActionPanel, List } from "@project-gauntlet/api/components";

export default function SearchBarSetSearchTextExample(): ReactElement {
    const [searchText, setSearchText] = useState<string>("");

    return (
        <List
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
            <List.SearchBar value={searchText} onChange={setSearchText}/>
            {/* This id will be the value in search bar when clicked or press enter when item is focused */}
            <List.Item id="item-focused" title="Click me!"/>
        </List>
    )
}
