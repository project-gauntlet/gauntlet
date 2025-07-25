import { Action, ActionPanel, IconAccessory, Icons, List, TextAccessory } from "@project-gauntlet/api/components";
import { ReactElement, useState } from "react";
import { Environment } from "@project-gauntlet/api/helpers";

export default function ListView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    const onClick = (id: string | null) => {
        if (id == "print-env") {
            console.log(Environment.gauntletVersion);
            console.log(Environment.isDevelopment);
            console.log(Environment.pluginCacheDir);
            console.log(Environment.pluginDataDir);
        } else {
            console.log("onClick " + id)
        }
    };

    const [searchText, setSearchText] = useState<string | undefined>("");

    return (
        <List
            actions={
                <ActionPanel>
                    <Action label="Run action" onAction={onClick}/>
                </ActionPanel>
            }>
            <List.SearchBar
                placeholder={"Search something..."}
                value={searchText}
                onChange={setSearchText}
            />
            <List.Item id="title" title={"Title"} subtitle={"Subtitle"}/>
            {
                numbers.map(value => {
                    const title = "Title " + value;

                    if (title.toLowerCase().includes(searchText?.toLowerCase() ?? "")) {
                        return (
                            <List.Item id={"title-" + value} key={"title-" + value} title={title}/>
                        )
                    } else {
                        return undefined
                    }
                })
            }
            <List.Section title={"Section"} subtitle="Test subtitle">
                <List.Section.Item id="title-section-1" title="Title Section 1" icon={Icons.Clipboard}/>
            </List.Section>
            <List.Section.Item id="print-env" title="Print environment" icon={Icons.Book}/>
            <List.Section title="Section 2">
                <List.Section.Item id="title-section-2-1" title="Title Section 2 1" subtitle="Subtitle 2 1"/>
                <List.Section.Item id="title-section-2-2" title="Title Section 2 2"/>
                <List.Section.Item
                    id="title-section-2-3"
                    title="Title Section 2 3"
                    subtitle="Subtitle 2 3"
                    accessories={[
                        <TextAccessory text="Accessory" icon={Icons.Alarm} tooltip={"Tooltip"}/>,
                        <IconAccessory icon={Icons.CloudSnow} tooltip={"Tooltip"}/>
                    ]}
                />
            </List.Section>
        </List>
    )
}
