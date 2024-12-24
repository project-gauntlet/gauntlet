import { IconAccessory, Icons, List, TextAccessory } from "@project-gauntlet/api/components";
import { ReactElement, useState } from "react";
import { Environment } from "@project-gauntlet/api/helpers";

export default function ListView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    const [id, setId] = useState("default");

    const onClick = () => {
        console.log("onClick " + id)
        setId(id);
    };

    const [searchText, setSearchText] = useState<string | undefined>("");

    return (
        <List>
            <List.SearchBar
                placeholder={"Search something..."}
                value={searchText}
                onChange={setSearchText}
            />
            <List.Item title={"Title"} subtitle={"Subtitle"} onClick={onClick}/>
            {
                numbers.map(value => {
                    const title = "Title " + value;

                    if (title.toLowerCase().includes(searchText?.toLowerCase() ?? "")) {
                        return (
                            <List.Item title={title} onClick={onClick}/>
                        )
                    } else {
                        return undefined
                    }
                })
            }
            <List.Section title={"Selected id: " + id} subtitle="Test subtitle">
                <List.Section.Item title="Title Section 1" icon={Icons.Clipboard} onClick={onClick}/>
            </List.Section>
            <List.Section.Item title="Print environment" icon={Icons.Book} onClick={() => {
                console.log(Environment.gauntletVersion);
                console.log(Environment.isDevelopment);
                console.log(Environment.pluginCacheDir);
                console.log(Environment.pluginDataDir);
            }}/>
            <List.Section title="Section 2">
                <List.Section.Item title="Title Section 2 1" subtitle="Subtitle 2 1" onClick={onClick}/>
                <List.Section.Item title="Title Section 2 2" onClick={onClick}/>
                <List.Section.Item
                    title="Title Section 2 3"
                    subtitle="Subtitle 2 3"
                    onClick={onClick}
                    accessories={[
                        <TextAccessory text="Accessory" icon={Icons.Alarm} tooltip={"Tooltip"}/>,
                        <IconAccessory icon={Icons.CloudSnow} tooltip={"Tooltip"}/>
                    ]}
                />
            </List.Section>
        </List>
    )
}