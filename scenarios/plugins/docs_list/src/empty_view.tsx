import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

async function readFile(): Promise<ArrayBuffer> {
    const res = await fetch(`https://static.wikia.nocookie.net/starwars/images/4/4a/Alderaan.jpg/revision/latest?cb=20061211013805`);
    const blob = await res.blob();
    return await blob.arrayBuffer()
}

const alderaanImage = await readFile()

export default function Main(): ReactElement {
    return (
        <List>
            <List.EmptyView title={"Nothing here"} description={"But there was something"} image={{ data: alderaanImage }}/>
        </List>
    )
}
