import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

async function readFile(url: string): Promise<ArrayBuffer> {
    const res = await fetch(url);
    const blob = await res.blob();
    return await blob.arrayBuffer()
}

const img = await readFile("https://static.wikia.nocookie.net/starwars/images/a/ae/The_Whills_Strike_Back.png/revision/latest/scale-to-width-down/400?cb=20201006180053")

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Image source={{ data: img }}/>
            </Detail.Content>
        </Detail>
    )
}
