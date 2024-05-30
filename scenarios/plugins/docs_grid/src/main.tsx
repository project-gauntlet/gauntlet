import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

async function readFile(url: string): Promise<ArrayBuffer> {
    const res = await fetch(url);
    const blob = await res.blob();
    return await blob.arrayBuffer()
}

const nabooImage = await readFile("https://static.wikia.nocookie.net/star-wars-canon/images/2/24/NabooFull-SW.png/revision/latest/scale-to-width-down/150?cb=20151218205422")
const rylothImage = await readFile("https://static.wikia.nocookie.net/star-wars-canon/images/4/48/Dagobah_ep3.jpg/revision/latest/scale-to-width-down/150?cb=20161103221846")
const tatooineImage = await readFile("https://static.wikia.nocookie.net/star-wars-canon/images/b/b7/Ryloth_Rebels.png/revision/latest/scale-to-width-down/150?cb=20161103040944")
const dagobahImage = await readFile("https://static.wikia.nocookie.net/star-wars-canon/images/b/b0/Tatooine_TPM.png/revision/latest/scale-to-width-down/150?cb=20151124205032")
const coruscantImage = await readFile("https://static.wikia.nocookie.net/star-wars-canon/images/7/7d/Death_Star_detail.png/revision/latest/scale-to-width-down/150?cb=20151216212148")
const endorImage = await readFile("https://static.wikia.nocookie.net/star-wars-canon/images/9/96/Endor-DB.png/revision/latest/scale-to-width-down/150?cb=20160711234205")
const deathstarImage = await readFile("https://static.wikia.nocookie.net/starwars/images/a/a6/Coruscant-SWJS.jpg/revision/latest/scale-to-width-down/150?cb=20240324185443")
const dathomirImage = await readFile("https://static.wikia.nocookie.net/starwars/images/3/34/DathomirJFO.jpg/revision/latest/scale-to-width-down/150?cb=20200222032237")
const dantooineImage = await readFile("https://static.wikia.nocookie.net/starwars/images/a/a5/Dantooine_Resistance.jpg/revision/latest/scale-to-width-down/150?cb=20200120190043")

export default function Main(): ReactElement {
    return (
        <Grid>
            <Grid.Item id="naboo" title="Naboo">
                <Grid.Item.Content.Image source={{data: nabooImage}}/>
            </Grid.Item>
            <Grid.Item id="ryloth" title="Ryloth">
                <Grid.Item.Content.Image source={{data: rylothImage}}/>
            </Grid.Item>
            <Grid.Item id="tatooine" title="Tatooine">
                <Grid.Item.Content.Image source={{data: tatooineImage}}/>
            </Grid.Item>
            <Grid.Item id="dagobah" title="Dagobah">
                <Grid.Item.Content.Image source={{data: dagobahImage}}/>
            </Grid.Item>
            <Grid.Item id="coruscant" title="Coruscant">
                <Grid.Item.Content.Image source={{data: coruscantImage}}/>
            </Grid.Item>
            <Grid.Item id="endor" title="Endor">
                <Grid.Item.Content.Image source={{data: endorImage}}/>
            </Grid.Item>
            <Grid.Item id="deathstar" title="Death Star">
                <Grid.Item.Content.Image source={{data: deathstarImage}}/>
            </Grid.Item>
            <Grid.Item id="dathomir" title="Dathomir">
                <Grid.Item.Content.Image source={{data: dathomirImage}}/>
            </Grid.Item>
            <Grid.Item id="dantooine" title="Dantooine">
                <Grid.Item.Content.Image source={{data: dantooineImage}}/>
            </Grid.Item>
        </Grid>
    )
}