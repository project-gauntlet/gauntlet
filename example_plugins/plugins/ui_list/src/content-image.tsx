import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

const imgUrl = "https://static.wikia.nocookie.net/starwars/images/e/e4/Ezaraa.png/revision/latest/scale-to-width-down/200?cb=20170511082800"

export default function ContentImageExample(): ReactElement {
    return (
        <List>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Detail>
                <List.Detail.Content>
                    <List.Detail.Content.Image source={{ url: imgUrl }}/>
                </List.Detail.Content>
            </List.Detail>
        </List>
    )
}
