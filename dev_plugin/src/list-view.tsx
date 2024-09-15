import { Icons, List } from "@project-gauntlet/api/components";
import { ReactElement, useState } from "react";

export default function ListView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    const [id, setId] = useState("default");

    const onClick = () => {
        console.log("onClick " + id)
        setId(id);
    };

    return (
        <List>
            {
                numbers.map(value => (
                    <List.Item title={"Title " + value} onClick={onClick}/>
                ))
            }
            <List.Section title={"Selected id: " + id}>
                <List.Section.Item title="Title Section 1" icon={Icons.Clipboard} onClick={onClick}/>
            </List.Section>
            <List.Section title="Section 2">
                <List.Section.Item title="Title Section 2 1" subtitle="Subtitle 2 1" onClick={onClick}/>
                <List.Section.Item title="Title Section 2 2" onClick={onClick}/>
                <List.Section.Item title="Title Section 2 3" subtitle="Subtitle 2 3" onClick={onClick}/>
            </List.Section>
        </List>
    )
}