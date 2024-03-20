import { List } from "@project-gauntlet/api/components";
import { ReactElement, useState } from "react";
import upperCase from "lodash/upperCase";

export default function ListView(): ReactElement {
    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    const [id, setId] = useState("default");

    return (
        <List
            onSelectionChange={id => {
                console.log("onSelectionChange " + id)
                if (id === undefined) { // FIXME this should never be undefined
                    throw new Error("not gonna happen")
                }
                setId(id);
            }}
        >
            {
                numbers.map(value => (
                    <List.Item id={"id" + value} title={"Title " + value}/>
                ))
            }
            <List.Section title="Section 1">
                <List.Section.Item id="id section 1" title="Title Section 1"/>
            </List.Section>
            <List.Section title="Section 2">
                <List.Section.Item id="id section 2 1" title="Title Section 2 1" subtitle="Subtitle 2 1"/>
                <List.Section.Item id="id section 2 2" title="Title Section 2 2"/>
                <List.Section.Item id="id section 2 3" title="Title Section 2 3" subtitle={"Selected id: " + id}/>
            </List.Section>
        </List>
    )
}