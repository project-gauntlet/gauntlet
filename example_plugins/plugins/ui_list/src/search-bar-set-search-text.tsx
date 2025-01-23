import { ReactElement, useState } from "react";
import { Action, ActionPanel, List } from "@project-gauntlet/api/components";

export default function SearchBarSetSearchTextExample(): ReactElement {
    const [searchText, setSearchText] = useState<string | undefined>("");

    return (
        <List
            actions={
                <ActionPanel>
                    <Action label="Set value" onAction={(id) => setSearchText(id)}/>
                </ActionPanel>
            }
        >
            <List.SearchBar value={searchText} onChange={setSearchText}/>
            <List.Item id="This will be the value in search bar" title="Click me!"/>
        </List>
    )
}
