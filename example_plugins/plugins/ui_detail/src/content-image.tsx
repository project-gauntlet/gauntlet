import { ReactNode } from "react";
import { Detail } from "@project-gauntlet/api/components";

const imgUrl = "https://static.wikia.nocookie.net/starwars/images/a/ae/The_Whills_Strike_Back.png/revision/latest/scale-to-width-down/250?cb=20201006180053"

export default function ContentImage(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Image source={{ url: imgUrl }}/>
            </Detail.Content>
        </Detail>
    )
}
